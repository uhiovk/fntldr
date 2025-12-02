use std::collections::HashSet;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

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
        #[allow(clippy::expect_used, reason = "explicit panic")]
        return dirs::cache_dir()
            .expect("Cache directory does not exist")
            .join(CACHE_DIR_NAME)
            .join(CACHE_FILENAME);
    };

    if path.is_file() {
        // input already points to a file
        path.to_owned()
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
            if current_dir_cache.is_file() { current_dir_cache } else { get_cache_path(None) }
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

// not parsing other styles because I'm lazy
pub fn parse_weight(name: &str) -> (&str, &str) {
    static FONT_WEIGHTS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
        // only consider some common weight names
        [
            // weights
            "extralight", "ultralight", "extrathin", "light", "thin", "demilight", "semilight",
            "book", "regular", "normal", "medium", "demibold", "semibold", "bold", "heavy",
            "black", "extrabold", "ultrabold",
        ]
        .into()
    });

    if let Some((family, weight)) = name.rsplit_once(" ")
        && FONT_WEIGHTS.contains(weight.to_ascii_lowercase().as_str())
    {
        (family, weight)
    } else {
        (name, "Regular")
    }
}

pub fn is_font(path: &Path) -> bool {
    ext_endswith(path, &["ttf", "otf", "ttc"])
}

pub fn is_ssa(path: &Path) -> bool {
    ext_endswith(path, &["ssa", "ass"])
}

fn ext_endswith(path: &Path, extensions: &[impl AsRef<str>]) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(ext) = path.extension() else {
        return false;
    };

    let ext = ext.to_ascii_lowercase();

    extensions.iter().any(|tgt| ext == tgt.as_ref())
}
