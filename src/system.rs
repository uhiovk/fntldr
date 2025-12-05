#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait FindFont {
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>>;
}

pub trait LoadFontFiles {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()>;
    fn unload_all(self);
}

pub struct LoaderWrapper<T: LoadFontFiles>(Option<T>);

impl<T: LoadFontFiles> LoaderWrapper<T> {
    fn new(loader: T) -> Self {
        Self(Some(loader))
    }

    pub fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        #[allow(clippy::unwrap_used, reason = "guaranteed `Some`")]
        self.0.as_mut().unwrap().load(files)
    }
}

impl<T: LoadFontFiles> Drop for LoaderWrapper<T> {
    fn drop(&mut self) {
        #[allow(clippy::unwrap_used, reason = "guaranteed `Some`")]
        self.0.take().unwrap().unload_all();
    }
}

pub fn get_finder() -> Result<impl FindFont> {
    #[cfg(target_os = "linux")]
    return Ok(self::linux::FontconfigFinder);

    #[cfg(target_os = "windows")]
    return Ok(self::windows::Finder);
}

pub fn get_loader() -> Result<LoaderWrapper<impl LoadFontFiles>> {
    #[cfg(target_os = "linux")]
    return Ok(LoaderWrapper::new(self::linux::FontconfigLoader::new()?));

    #[cfg(target_os = "windows")]
    return Ok(LoaderWrapper::new(self::windows::Loader::new()));
}
