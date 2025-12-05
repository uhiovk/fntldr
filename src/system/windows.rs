use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::Graphics::Gdi::{
    AddFontResourceW, EnumFontFamiliesExW, GetDC, LOGFONTW, ReleaseDC, RemoveFontResourceW,
    TEXTMETRICW,
};

use crate::system::{FindFont, LoadFontFiles};
use crate::utils::parse_style;

pub struct Finder;

impl FindFont for Finder {
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>> {
        unsafe extern "system" fn callback(
            _logfont: *const LOGFONTW,
            _metrics: *const TEXTMETRICW,
            _font_type: u32,
            found_match: LPARAM,
        ) -> i32 {
            unsafe {
                *(found_match as *mut bool) = true;
            }
            0
        }

        let (family, style) = parse_style(name.as_ref());
        let face_name =
            if style == "Regular" { family.to_owned() } else { family.to_owned() + " " + style };
        let mut name_utf16 = face_name.encode_utf16();
        let name_utf16_arr = std::array::from_fn(|_| name_utf16.next().unwrap_or(0));

        let hdc = unsafe { GetDC(std::ptr::null_mut()) };
        let logfont = LOGFONTW { lfFaceName: name_utf16_arr, ..Default::default() };
        let mut found_match = false;

        unsafe {
            EnumFontFamiliesExW(
                hdc,
                &logfont,
                Some(callback),
                &mut found_match as *mut _ as LPARAM,
                0,
            );
            ReleaseDC(std::ptr::null_mut(), hdc);
        }

        if found_match { Ok(Some(PathBuf::new())) } else { Ok(None) }
    }
}

pub struct Loader {
    loaded: Vec<Vec<u16>>,
}

impl Loader {
    pub fn new() -> Self {
        Self { loaded: Vec::new() }
    }
}

impl LoadFontFiles for Loader {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        for file in files {
            let path_utf16: Vec<_> = file.as_ref().as_os_str().encode_wide().chain([0]).collect();
            if unsafe { AddFontResourceW(path_utf16.as_ptr()) } == 0 {
                bail!("AddFontResource failed");
            }
            self.loaded.push(path_utf16);
        }

        Ok(())
    }

    fn unload_all(self) {
        for path_utf16 in &self.loaded {
            if unsafe { RemoveFontResourceW(path_utf16.as_ptr()) } == 0 {
                eprintln!("Failed to unregister file: {}", String::from_utf16_lossy(path_utf16));
            }
        }
    }
}
