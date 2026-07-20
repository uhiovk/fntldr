use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub fn walk_dir(
    path: &Path,
    is_recursive: bool,
    may_process: &impl Fn(&Path) -> bool,
    callback: &mut impl FnMut(PathBuf),
) {
    let Ok(rd) = read_dir(path) else { return };
    for entry in rd {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if may_process(&path) {
            callback(path);
        } else if is_recursive && path.is_dir() {
            walk_dir(&path, true, may_process, callback);
        }
    }
}

pub fn get_db_path(path: Option<&Path>) -> PathBuf {
    if let Some(path) = path {
        if path.is_file() { path.to_owned() } else { path.join("fntldr.bin") }
    } else {
        let current_dir_db = PathBuf::from("./fntldr.bin");
        if current_dir_db.is_file() {
            current_dir_db
        } else {
            dirs::cache_dir().unwrap().join("fntldr/fntldr.bin")
        }
    }
}

pub fn parse_style(name: &str) -> (&str, &str) {
    // only consider some common style names
    static STYLES: [&str; 16] = [
        "thin", "extralight", "ultralight", "light", "regular", "normal", "medium", "semibold",
        "demibold", "bold", "extrabold", "ultrabold", "heavy", "black", "italic", "oblique",
    ];

    let mut word_start = name.len();

    for (idx, _) in name.rmatch_indices(' ') {
        let word = &name[idx + 1..word_start];
        if word.is_empty() || STYLES.contains(&word.to_ascii_lowercase().as_str()) {
            word_start = idx;
        } else {
            break;
        }
    }

    if word_start == name.len() {
        (name, "Regular")
    } else {
        let (family, style) = name.split_at(word_start);
        (family, style.trim_ascii_start())
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
