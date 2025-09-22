#[cfg(target_os = "linux")]
mod linux;

use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait FindFont {
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>>;
}

// should implement `Drop` to unload fonts on drop
pub trait LoadFontFiles {
    fn load(
        &mut self,
        files: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Result<()>;
}

pub fn get_finder() -> Result<impl FindFont> {
    #[cfg(target_os = "linux")]
    Ok(linux::FontconfigFinder)
}

pub fn get_loader() -> Result<impl LoadFontFiles> {
    #[cfg(target_os = "linux")]
    linux::FontconfigLoader::new()
}
