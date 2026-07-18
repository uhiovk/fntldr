use std::ffi::{CStr, CString, OsStr};
use std::fs::{remove_dir_all, remove_file};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::ptr::{self, NonNull};

use anyhow::Result;
use fontconfig_sys::constants::{FC_FAMILY, FC_FILE, FC_STYLE};
use fontconfig_sys::{
    FcConfigBuildFonts, FcConfigSubstitute, FcDefaultSubstitute, FcDirCacheRead, FcFontMatch,
    FcMatchPattern, FcPattern, FcPatternAddString, FcPatternCreate, FcPatternDestroy,
    FcPatternGetString, FcResultMatch,
};
use tempfile::{TempDir, tempdir};

use super::{FindFont, LoadFont};
use crate::utils::parse_style;

pub struct FontconfigFinder;

// It is impossible to determine whether a returned pattern is the result
// of default substitution or user config substitution (usually aliases)
// when none of the family names in it matches exactly with the input,
// while user config itself might also substitute to an invalid family name.
// The function only checks if there's an exact match, it cannot properly
// handle aliases, so it might return `false` even if the inteded font is
// actually installed, resulting in loading unnecessary fonts.
// For example, you'd probably expect `true` for "sans-serif" or "monospace",
// but it'll return the opposite, since no actual font would have the name.
impl FindFont for FontconfigFinder {
    fn get_font_file(&self, name: &str) -> Result<Option<PathBuf>> {
        let (family, style) = parse_style(name);

        let mut pattern = Pattern::new();
        pattern.add_string(FC_FAMILY, &CString::new(family)?);
        pattern.add_string(FC_STYLE, &CString::new(style)?);
        pattern.config_sub(FcMatchPattern);
        pattern.default_sub();

        let font_match = pattern.match_font();
        if !font_match.has_family(family) {
            return Ok(None);
        }
        Ok(font_match.file())
    }
}

pub struct FontconfigLoader {
    _tmpdir: TempDir,
    link: PathBuf,
}

impl FontconfigLoader {
    pub fn new() -> Result<Self> {
        let _tmpdir = tempdir()?;
        let link = dirs::font_dir().unwrap().join(".fntldrtmp");

        if link.is_symlink() {
            if link.is_dir() {
                // already a valid link
                return Ok(Self { _tmpdir, link });
            } else {
                // link is broken
                remove_file(&link)?;
            }
        }

        symlink(_tmpdir.path(), &link)?;
        Ok(Self { _tmpdir, link })
    }
}

impl LoadFont for FontconfigLoader {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        for file in files {
            let file = file.as_ref();
            let target = self.link.join(file.file_name().unwrap());
            symlink(file, &target)?;
        }

        let c_dir = CString::new(self.link.as_os_str().as_bytes())?;
        unsafe {
            // it's like `fc-cache -f` on a single directory
            FcDirCacheRead(c_dir.as_ptr() as *const u8, 1, ptr::null_mut());
        }

        Ok(())
    }
}

impl Drop for FontconfigLoader {
    fn drop(&mut self) {
        if remove_dir_all(&self.link).is_err() {
            eprintln!("cannot remove symlink \"{}\"", self.link.display());
        }

        if unsafe { FcConfigBuildFonts(ptr::null_mut()) } == 0 {
            eprintln!("FcConfigBuildFonts failed");
            eprintln!("Please run `fc-cache` yourself");
        }
    }
}

struct Pattern(NonNull<FcPattern>);

impl Pattern {
    fn new() -> Self {
        Self::new_from(unsafe { FcPatternCreate() }).unwrap()
    }

    fn new_from(ptr: *mut FcPattern) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    fn add_string(&mut self, key: &CStr, value: &CStr) {
        assert_eq!(
            unsafe { FcPatternAddString(self.0.as_ptr(), key.as_ptr(), value.as_ptr() as _) },
            0
        );
    }

    fn default_sub(&mut self) {
        unsafe {
            FcDefaultSubstitute(self.0.as_ptr());
        }
    }

    fn config_sub(&mut self, match_kind: u32) {
        assert_eq!(unsafe { FcConfigSubstitute(ptr::null_mut(), self.0.as_ptr(), match_kind) }, 0)
    }

    fn match_font(&self) -> Self {
        let mut result = 0;
        let ptr = unsafe { FcFontMatch(ptr::null_mut(), self.0.as_ptr(), &mut result) };
        assert_eq!(result, 0);
        Self::new_from(ptr).unwrap()
    }

    fn file(&self) -> Option<PathBuf> {
        let mut match_res_ptr = ptr::null_mut();
        let res =
            unsafe { FcPatternGetString(self.0.as_ptr(), FC_FILE.as_ptr(), 0, &mut match_res_ptr) };
        if res != FcResultMatch || match_res_ptr.is_null() {
            return None;
        }

        let path = unsafe { CString::from_raw(match_res_ptr as _) };
        Some(OsStr::from_bytes(path.as_bytes()).to_owned().into())
    }

    fn has_family(&self, family: &str) -> bool {
        let family = family.to_ascii_lowercase();
        for i in 0.. {
            let mut match_res_ptr = ptr::null_mut();
            let res = unsafe {
                FcPatternGetString(self.0.as_ptr(), FC_FAMILY.as_ptr(), i, &mut match_res_ptr)
            };
            if res != FcResultMatch {
                break;
            }
            if match_res_ptr.is_null() {
                continue;
            }

            let name = unsafe { CString::from_raw(match_res_ptr as _) }
                .to_string_lossy()
                .to_ascii_lowercase();
            if name == family {
                return true;
            }
        }
        false
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            FcPatternDestroy(self.0.as_ptr());
        }
    }
}
