use anyhow::{Context, Result};
use bincode::{
    Decode, Encode, config::standard, decode_from_std_read,
    encode_into_std_write,
};
use memmap2::Mmap;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::{File, create_dir_all, read_dir};
use std::path::{Path, PathBuf, absolute};
use std::sync::LazyLock;
use ttf_parser::{Face, fonts_in_collection, name_id::FULL_NAME};

#[derive(Encode, Decode)]
struct FontFile {
    path: PathBuf,
    names: Vec<String>,
    is_variable: bool,
}

#[derive(Encode, Decode)]
pub struct FontProviders {
    files: Vec<FontFile>,
    map: HashMap<String, usize>,
}

impl FontProviders {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path).with_context(|| {
            format!("Error opening file \"{}\"", path.display())
        })?;

        decode_from_std_read(&mut file, standard()).with_context(|| {
            format!("Error reading file \"{}\"", path.display())
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            create_dir_all(dir).with_context(|| {
                format!("Error creating directory \"{}\"", path.display())
            })?;
        }

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .with_context(|| {
                format!("Error opening file \"{}\"", path.display())
            })?;

        encode_into_std_write(self, &mut file, standard()).with_context(
            || format!("Error writing file \"{}\"", path.display()),
        )?;

        Ok(())
    }

    pub fn index(
        &mut self,
        path: &Path,
        is_recursive: bool,
        is_absolute: bool,
    ) {
        let path = if is_absolute {
            Cow::from(absolute(path).unwrap())
        } else {
            Cow::from(path)
        };

        let Ok(entries) = read_dir(&path) else {
            eprintln!("Error reading directory \"{}\"", path.display());
            return;
        };

        for entry in entries {
            let Ok(entry) = entry else {
                eprintln!("Error reading directory \"{}\"", path.display());
                continue;
            };

            let path = entry.path();

            if is_font(&path) {
                let (names, is_variable) = Self::get_font_names(&path);
                let idx = self.files.len();
                self.map
                    .extend(names.iter().cloned().map(|name| (name, idx)));
                self.files.push(FontFile {
                    path,
                    names,
                    is_variable,
                });
            } else if is_recursive && path.is_dir() {
                self.index(&path, is_recursive, is_absolute);
            }
        }
    }

    pub fn file_by_font_name(&self, name: &str) -> Option<&PathBuf> {
        if let Some(file_idx) = self.map.get(name) {
            return Some(&self.files[*file_idx].path);
        }

        // match variable fonts, only by family name
        // assuming variable fonts provide any weight
        let (family, _) = parse_weight(name);
        if let Some(file_idx) = self.map.get(family) {
            let file = &self.files[*file_idx];
            if file.is_variable {
                return Some(&file.path);
            }
        }

        None
    }

    fn get_font_names(path: &Path) -> (Vec<String>, bool) {
        let Ok(file) = File::open(path) else {
            eprintln!("Error reading file \"{}\"", path.display());
            return (Vec::new(), false);
        };

        // memmap so we don't have to read the whole file
        let Ok(mapped) = (unsafe { Mmap::map(&file) }) else {
            eprintln!("Error reading file \"{}\"", path.display());
            return (Vec::new(), false);
        };

        // would return None for a regular font file (.ttf / .otf)
        let num_faces = fonts_in_collection(&mapped).unwrap_or(1);

        let mut is_variable = false;

        let names = (0..num_faces)
            .filter_map(|i| Face::parse(&mapped, i).ok())
            // no sane people would put variable and
            // non-variable fonts in a single collection
            .inspect(|face| is_variable = face.is_variable())
            .flat_map(|face| face.names())
            .filter(|name| name.name_id == FULL_NAME)
            .filter_map(|name| {
                // try UTF-16 first
                name.to_string().or_else(
                    // then try UTF-8
                    || String::from_utf8(name.name.to_vec()).ok(),
                )
            })
            .collect();

        (names, is_variable)
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

pub fn is_font(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(ext) = path.extension() else {
        return false;
    };

    let ext = ext.to_ascii_lowercase();

    ext == "ttf" || ext == "otf" || ext == "ttc"
}

// not parsing other styles because I'm lazy
pub fn parse_weight(name: &str) -> (&str, &str) {
    if let Some((family, weight)) = name.rsplit_once(" ")
        && FONT_WEIGHTS.contains(weight.to_ascii_lowercase().as_str())
    {
        (family, weight)
    } else {
        (name, "Regular")
    }
}

// only consider some common weight names
static FONT_WEIGHTS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    let mut set = HashSet::with_capacity(18);

    set.insert("extralight");
    set.insert("ultralight");
    set.insert("extrathin");
    set.insert("light");
    set.insert("thin");
    set.insert("demilight");
    set.insert("semilight");
    set.insert("book");
    set.insert("regular");
    set.insert("normal");
    set.insert("medium");
    set.insert("demibold");
    set.insert("semibold");
    set.insert("bold");
    set.insert("heavy");
    set.insert("black");
    set.insert("extrabold");
    set.insert("ultrabold");

    set
});
