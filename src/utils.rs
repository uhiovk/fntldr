use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub fn walk_dir(
    path: &Path,
    is_recursive: bool,
    may_process: &impl Fn(&Path) -> bool,
    process: &mut impl FnMut(PathBuf),
) {
    // report and ignore errors
    let Ok(entries) = read_dir(path) else {
        eprintln!("Error reading directory \"{}\"", path.display());
        return;
    };

    for entry in entries {
        let Ok(entry) = entry else {
            eprintln!("Error reading directory \"{}\"", path.display());
            continue;
        };

        let path = entry.path();

        if may_process(&path) {
            process(path);
        } else if is_recursive && path.is_dir() {
            walk_dir(&path, true, may_process, process);
        }
    }
}

pub fn get_cache_path(path: Option<&Path>) -> PathBuf {
    const CACHE_DIR_NAME: &str = "fntldr";
    const CACHE_FILENAME: &str = "fntldr_cache.bin";

    let Some(path) = path else {
        // not specified, use default cache directory
        return dirs::cache_dir()
            .expect("Cache directory does not exist")
            .join(CACHE_DIR_NAME)
            .join(CACHE_FILENAME);
    };

    if path.is_file() {
        // input already points to a file
        path.to_path_buf()
    } else {
        // assume input points to a directory
        path.join(CACHE_FILENAME)
    }
}

pub fn get_cache_path_fallback(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) => get_cache_path(Some(path)),
        None => {
            let current_dir_cache = get_cache_path(Some(&PathBuf::from(".")));
            if current_dir_cache.is_file() {
                current_dir_cache
            } else {
                get_cache_path(None)
            }
        }
    }
}

pub fn get_font_list_path(path: Option<&Path>) -> PathBuf {
    const DEFAULT_LOCATION: &str = "./fonts.txt";

    if path.is_some() {
        unimplemented!();
    }

    PathBuf::from(DEFAULT_LOCATION)
}

pub fn is_font(path: &Path) -> bool {
    const FONT_FILE_EXTS: [&str; 3] = ["ttf", "otf", "ttc"];
    ext_endswith(path, FONT_FILE_EXTS)
}

pub fn is_ssa(path: &Path) -> bool {
    const SSA_FILE_EXTS: [&str; 2] = ["ssa", "ass"];
    ext_endswith(path, SSA_FILE_EXTS)
}

fn ext_endswith<const N: usize>(path: &Path, extensions: [&str; N]) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(ext) = path.extension() else {
        return false;
    };

    let ext = ext.to_ascii_lowercase();

    extensions.into_iter().any(|tgt| ext == tgt)
}
