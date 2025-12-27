#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use std::path::{Path, PathBuf};

use anyhow::Result;

#[cfg(target_os = "linux")]
type FinderImpl = self::linux::FontconfigFinder;

#[cfg(target_os = "windows")]
type FinderImpl = self::windows::Finder;

pub struct Finder(FinderImpl);

impl Finder {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        return Ok(Self(self::linux::FontconfigFinder));

        #[cfg(target_os = "windows")]
        return Ok(Self(self::windows::Finder));
    }

    pub fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>> {
        self.0.get_font_file(name)
    }
}

#[cfg(target_os = "linux")]
type LoaderImpl = self::linux::FontconfigLoader;

#[cfg(target_os = "windows")]
type LoaderImpl = self::windows::Loader;

pub struct Loader(Option<LoaderImpl>);

impl Loader {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        return Ok(Self(Some(self::linux::FontconfigLoader::new()?)));

        #[cfg(target_os = "windows")]
        return Ok(Self(Some(self::windows::Loader::new())));
    }

    pub fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        #[allow(clippy::unwrap_used, reason = "guaranteed `Some`")]
        self.0.as_mut().unwrap().load(files)
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        #[allow(clippy::unwrap_used, reason = "guaranteed `Some`")]
        self.0.take().unwrap().unload_all();
    }
}

trait FindFont {
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>>;
}

trait LoadFontFiles {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()>;
    fn unload_all(self);
}
