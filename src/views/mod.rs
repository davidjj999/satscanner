pub mod globe_bands;
pub mod globe_scale;
pub mod overhead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overhead,
    Sky,
    GlobeBands,
}
