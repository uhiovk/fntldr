use std::ffi::{CStr, CString, OsStr};
use std::fs::{remove_dir_all, remove_file};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::ptr;

use anyhow::{Context, Result, bail, ensure};
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

        unsafe {
            // create the pattern
            let pattern = Pattern(FcPatternCreate());

            ensure!(!pattern.0.is_null(), "FcPatternCreate failed");

            // add family name and style to the pattern
            if FcPatternAddString(
                pattern.0,
                FC_FAMILY.as_ptr(),
                CString::new(family)?.as_ptr() as *const u8,
            ) == 0
            {
                bail!("FcPatternAddString failed");
            }

            if FcPatternAddString(
                pattern.0,
                FC_STYLE.as_ptr(),
                CString::new(style)?.as_ptr() as *const u8,
            ) == 0
            {
                bail!("FcPatternAddString failed");
            }

            // perform substitutions
            if FcConfigSubstitute(ptr::null_mut(), pattern.0, FcMatchPattern) == 0 {
                bail!("FcConfigSubstitute failed");
            };
            FcDefaultSubstitute(pattern.0);

            // match the pattern, basically equivalent to `fc-match`
            let font_match = Pattern(FcFontMatch(ptr::null_mut(), pattern.0, &mut 0));

            ensure!(!font_match.0.is_null(), "FcFontMatch failed");

            // check all family names of the returned best match
            let is_exact = font_match.get_families().contains(&family.to_ascii_lowercase());

            if !is_exact {
                return Ok(None);
            }

            let path = font_match.get_file()?;

            Ok(Some(path))
        }
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
                remove_file(&link).with_context(|| {
                    format!("cannot remove broken symlink \"{}\"", link.display())
                })?;
            }
        }

        symlink(_tmpdir.path(), &link).with_context(|| {
            format!("cannot link from \"{}\" to \"{}\"", _tmpdir.path().display(), link.display())
        })?;

        Ok(Self { _tmpdir, link })
    }
}

impl LoadFont for FontconfigLoader {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        for file in files {
            let file = file.as_ref();
            let target = self.link.join(file.file_name().unwrap());
            symlink(file, &target).with_context(|| {
                format!("cannot link from \"{}\" to \"{}\"", file.display(), target.display())
            })?;
        }

        let c_dir = CString::new(self.link.as_os_str().as_bytes())?;
        unsafe {
            // it's like `fc-cache -f` on a single directory
            FcDirCacheRead(c_dir.as_ptr() as *const u8, 1, ptr::null_mut());
        }

        Ok(())
    }

    fn unload_all(self) {
        if remove_dir_all(&self.link).is_err() {
            eprintln!("cannot remove symlink \"{}\"", self.link.display());
        }

        if unsafe { FcConfigBuildFonts(ptr::null_mut()) } == 0 {
            eprintln!("FcConfigBuildFonts failed");
            eprintln!("Please run `fc-cache` yourself");
        }
    }
}

struct Pattern(*mut FcPattern);

impl Pattern {
    fn get_file(&self) -> Result<PathBuf> {
        let mut match_res_ptr = ptr::null_mut();
        let result = unsafe { FcPatternGetString(self.0, FC_FILE.as_ptr(), 0, &mut match_res_ptr) };

        ensure!(result == FcResultMatch, "FcPatternGetString failed");
        ensure!(!match_res_ptr.is_null(), "FcPatternGetString failed");

        let path = unsafe { CStr::from_ptr(match_res_ptr as *const i8) };
        let path = OsStr::from_bytes(path.to_bytes());
        let path = Path::new(path).to_owned();

        Ok(path)
    }

    fn get_families(&self) -> Vec<String> {
        let mut families = Vec::new();

        for i in 0.. {
            let mut match_res_ptr = ptr::null_mut();

            if unsafe { FcPatternGetString(self.0, FC_FAMILY.as_ptr(), i, &mut match_res_ptr) }
                != FcResultMatch
            {
                break;
            }

            if match_res_ptr.is_null() {
                continue;
            }

            let name = unsafe { CStr::from_ptr(match_res_ptr as *const i8) }
                .to_string_lossy()
                .to_ascii_lowercase();

            families.push(name);
        }

        families
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            FcPatternDestroy(self.0);
        }
    }
}
