#[cfg(target_os = "linux")]
mod linux;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait FindFont {
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>>;
}

/// # Safety
/// should implement `Drop` to unload fonts on drop
pub unsafe trait LoadFontFiles {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()>;
}

pub fn get_finder() -> Result<impl FindFont> {
    #[cfg(target_os = "linux")]
    Ok(self::linux::FontconfigFinder)
}

pub fn get_loader() -> Result<impl LoadFontFiles> {
    #[cfg(target_os = "linux")]
    self::linux::FontconfigLoader::new()
}
