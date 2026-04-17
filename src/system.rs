#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait FindFont {
    fn get_font_file(&self, name: &str) -> Result<Option<PathBuf>>;
}

pub trait LoadFont {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()>;
}

#[cfg(target_os = "linux")]
use self::linux::{FontconfigFinder as SysFinder, FontconfigLoader as SysLoader};
#[cfg(target_os = "windows")]
use self::windows::{Finder as SysFinder, Loader as SysLoader};

pub struct Finder(SysFinder);

impl Finder {
    pub fn new() -> Result<Self> {
        Ok(Self(SysFinder))
    }
}

impl FindFont for Finder {
    fn get_font_file(&self, name: &str) -> Result<Option<PathBuf>> {
        self.0.get_font_file(name)
    }
}

pub struct Loader(SysLoader);

impl Loader {
    #[cfg(target_os = "linux")]
    pub fn new() -> Result<Self> {
        Ok(Self(SysLoader::new()?))
    }

    #[cfg(target_os = "windows")]
    pub fn new() -> Result<Self> {
        Ok(Self(SysLoader::new()))
    }
}

impl LoadFont for Loader {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        self.0.load(files)
    }
}
