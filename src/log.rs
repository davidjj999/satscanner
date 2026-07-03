use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const MAX_LOG_ENTRIES: usize = 500;
const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1 MB before rotation

// ---------------------------------------------------------------------------
// In-memory ring buffer (for the `l` key overlay)
// ---------------------------------------------------------------------------

/// Thread-safe ring buffer for in-app log messages captured from tracing.
#[derive(Clone)]
pub struct SharedLog(Arc<Mutex<LogStore>>);

struct LogStore {
    entries: VecDeque<(String, String)>, // (level, message)
}

impl SharedLog {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(LogStore {
            entries: VecDeque::with_capacity(MAX_LOG_ENTRIES),
        })))
    }

    pub fn push(&self, level: String, message: String) {
        if let Ok(mut store) = self.0.lock() {
            if store.entries.len() >= MAX_LOG_ENTRIES {
                store.entries.pop_front();
            }
            store.entries.push_back((level, message));
        }
    }

    pub fn entries(&self) -> Vec<(String, String)> {
        self.0
            .lock()
            .map(|store| store.entries.iter().cloned().collect())
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Rolling file log
// ---------------------------------------------------------------------------

/// Manages append-only writes to `satscanner.log` with automatic rotation.
/// Keeps at most one backup (`satscanner.log.1`).
#[derive(Clone)]
pub struct SharedLogFile(Arc<Mutex<FileLogInner>>);

struct FileLogInner {
    file: File,
    size: u64,
    path: PathBuf,
}

impl SharedLogFile {
    /// Open (or create) the log file at `path`, positioned at the end for
    /// appending.  File size is tracked so we know when to rotate.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("Failed to open log file for writing");
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        Self(Arc::new(Mutex::new(FileLogInner { file, size, path })))
    }

    /// Append one line (with trailing newline) to the file.  If the file has
    /// grown past `MAX_FILE_SIZE`, rotate: rename `path` → `path.1`, then
    /// start a fresh file.
    fn write_line(&self, line: &str) {
        if let Ok(mut inner) = self.0.lock() {
            let bytes = line.as_bytes();
            if inner.file.write_all(bytes).is_ok()
                && inner.file.write_all(b"\n").is_ok()
                && inner.file.flush().is_ok()
            {
                inner.size += (bytes.len() + 1) as u64;
                if inner.size > MAX_FILE_SIZE {
                    inner.rotate();
                }
            }
        }
    }
}

impl FileLogInner {
    fn rotate(&mut self) {
        // Flush and close the current handle, then rename.
        let _ = self.file.flush();
        let backup = self.path.with_extension("log.1");
        if backup.exists() {
            let _ = fs::remove_file(&backup);
        }
        if fs::rename(&self.path, &backup).is_ok() {
            // Open a fresh file.
            if let Ok(new_file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                self.file = new_file;
                self.size = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// tracing-subscriber writer plumbing
// ---------------------------------------------------------------------------

/// A custom [`std::io::Write`] that splits incoming formatted log output into
/// lines and forwards each one to the in-memory ring buffer **and** the
/// rolling file.
pub struct LogWriter {
    log: SharedLog,
    log_file: Option<SharedLogFile>,
    buf: Vec<u8>,
}

impl LogWriter {
    fn new(log: SharedLog, log_file: Option<SharedLogFile>) -> Self {
        Self {
            log,
            log_file,
            buf: Vec::new(),
        }
    }
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        // Flush on newline boundaries so each log line is one entry.
        while let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
            let raw = self.buf.drain(..=pos).collect::<Vec<_>>();
            let line =
                String::from_utf8_lossy(&raw[..raw.len().saturating_sub(1)]) // strip \n
                    .trim()
                    .to_string();
            if line.is_empty() {
                continue;
            }

            // Determine log level from the formatted line.
            // Default tracing format: `2024-01-01T12:00:00.123Z  INFO module: msg`
            let level = if line.contains(" ERROR ") {
                "ERROR"
            } else if line.contains(" WARN ") {
                "WARN"
            } else if line.contains(" INFO ") {
                "INFO"
            } else if line.contains(" DEBUG ") {
                "DEBUG"
            } else {
                "TRACE"
            };

            // Push to in-memory ring buffer.
            self.log.push(level.to_string(), line.clone());

            // Append to rolling file.
            if let Some(ref log_file) = self.log_file {
                log_file.write_line(&line);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// [`tracing_subscriber::fmt::MakeWriter`] implementation that feeds into the
/// in-memory ring buffer **and** (optionally) a rolling file on disk.
pub struct LogMakeWriter {
    log: SharedLog,
    log_file: Option<SharedLogFile>,
}

impl LogMakeWriter {
    pub fn new(log: SharedLog) -> Self {
        Self {
            log,
            log_file: None,
        }
    }

    /// Enable file output.  Call **before** passing to the tracing subscriber.
    pub fn with_file(mut self, log_file: SharedLogFile) -> Self {
        self.log_file = Some(log_file);
        self
    }
}

impl<'writer> tracing_subscriber::fmt::MakeWriter<'writer> for LogMakeWriter {
    type Writer = LogWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        LogWriter::new(self.log.clone(), self.log_file.clone())
    }
}