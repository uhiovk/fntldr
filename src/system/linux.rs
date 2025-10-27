use anyhow::{Context, Result, bail, ensure};
use fontconfig_sys::{
    FcConfigBuildFonts, FcConfigSubstitute, FcDefaultSubstitute, FcDirCacheRead, FcFontMatch,
    FcMatchPattern, FcPattern, FcPatternAddString, FcPatternCreate, FcPatternDestroy,
    FcPatternGetString, FcResultMatch,
    constants::{FC_FAMILY, FC_FILE, FC_STYLE},
};
use std::ffi::{CStr, CString, OsStr};
use std::fs::{remove_dir_all, remove_file};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::ptr;
use tempfile::{TempDir, tempdir};

use super::{FindFont, LoadFontFiles};
use crate::utils::parse_weight;

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
    fn get_font_file(&self, name: impl AsRef<str>) -> Result<Option<PathBuf>> {
        let (family, style) = parse_weight(name.as_ref());

        unsafe {
            // create the pattern
            let pattern = FcPatternPtr(FcPatternCreate());

            ensure!(
                !pattern.0.is_null(),
                "FcPatternCreate returned null pointer"
            );

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
            let font_match = FcPatternPtr(FcFontMatch(ptr::null_mut(), pattern.0, &mut 0));

            ensure!(!font_match.0.is_null(), "FcFontMatch returned null pointer");

            // check all family names of the returned best match
            let is_exact = families_in_pattern(&font_match).contains(&family.to_ascii_lowercase());

            if !is_exact {
                return Ok(None);
            }

            let path = file_in_pattern(&font_match)?;

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
        let link = dirs::font_dir()
            .expect("Fonts directory does not exist")
            .join(".fntldrtmp");

        if link.is_symlink() {
            if link.is_dir() {
                // already a valid link
                return Ok(Self { _tmpdir, link });
            } else {
                // link is broken
                remove_file(&link).with_context(|| {
                    format!("Error removing broken symlink \"{}\"", link.display())
                })?;
            }
        }

        symlink(_tmpdir.path(), &link).with_context(|| {
            format!(
                "Error linking from \"{}\" to \"{}\"",
                _tmpdir.path().display(),
                link.display()
            )
        })?;

        Ok(Self { _tmpdir, link })
    }
}

impl LoadFontFiles for FontconfigLoader {
    fn load(&mut self, files: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<()> {
        for file in files {
            let file = file.as_ref();
            let target = self.link.join(file.file_name().unwrap());
            symlink(file, &target).with_context(|| {
                format!(
                    "Error linking from \"{}\" to \"{}\"",
                    file.display(),
                    target.display()
                )
            })?;
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
            eprintln!("Error removing symlink \"{}\"", self.link.display());
        }

        let result = unsafe { FcConfigBuildFonts(ptr::null_mut()) };

        if result == 0 {
            eprintln!("FcConfigBuildFonts failed");
            eprintln!("Please run `fc-cache` yourself");
        }
    }
}

struct FcPatternPtr(*mut FcPattern);

impl Drop for FcPatternPtr {
    fn drop(&mut self) {
        unsafe {
            FcPatternDestroy(self.0);
        }
    }
}

unsafe fn file_in_pattern(pattern: &FcPatternPtr) -> Result<PathBuf> {
    let mut match_res_ptr = ptr::null_mut();

    let result = unsafe { FcPatternGetString(pattern.0, FC_FILE.as_ptr(), 0, &mut match_res_ptr) };

    ensure!(
        result == FcResultMatch,
        "FcPatternGetString failed, \
        no such field \"file\" in pattern"
    );

    ensure!(
        !match_res_ptr.is_null(),
        "FcPatternGetString returned null pointer"
    );

    let path = unsafe { CStr::from_ptr(match_res_ptr as *const i8) };
    let path = OsStr::from_bytes(path.to_bytes());
    let path = Path::new(path).to_owned();

    Ok(path)
}

unsafe fn families_in_pattern(pattern: &FcPatternPtr) -> Vec<String> {
    (0..)
        .map_while(|i| {
            let mut match_res_ptr = ptr::null_mut();

            (unsafe { FcPatternGetString(pattern.0, FC_FAMILY.as_ptr(), i, &mut match_res_ptr) }
                != FcResultMatch)
                .then_some(match_res_ptr)
        })
        .filter(|match_res_ptr| match_res_ptr.is_null())
        .filter_map(|match_res_ptr| {
            unsafe { CStr::from_ptr(match_res_ptr as *const i8) }
                .to_str()
                .map(|s| s.to_ascii_lowercase())
                .ok()
        })
        .collect()
}
