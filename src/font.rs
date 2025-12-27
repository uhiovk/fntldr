use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf, absolute};

use anyhow::{Context, Result};
use bincode::config::standard;
use bincode::{Decode, Encode, decode_from_std_read, encode_into_std_write};
use memmap2::Mmap;
use ttf_parser::name_id::FULL_NAME;
use ttf_parser::{Face, fonts_in_collection};

use crate::utils::{is_font, parse_style, walk_dir};

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
        Self { files: Vec::new(), map: HashMap::new() }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .with_context(|| format!("Error opening file \"{}\"", path.display()))?;

        decode_from_std_read(&mut file, standard())
            .with_context(|| format!("Error reading file \"{}\"", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            create_dir_all(dir)
                .with_context(|| format!("Error creating directory \"{}\"", path.display()))?;
        }

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("Error opening file \"{}\"", path.display()))?;

        encode_into_std_write(self, &mut file, standard())
            .with_context(|| format!("Error writing file \"{}\"", path.display()))?;

        Ok(())
    }

    pub fn index(&mut self, path: &Path, is_recursive: bool) {
        let mut process = |path: PathBuf| {
            let (names, is_variable) = Self::get_font_names(&path);
            let idx = self.files.len();
            self.map.extend(names.iter().cloned().map(|name| (name, idx)));
            self.files.push(FontFile { path, names, is_variable });
        };

        walk_dir(path, is_recursive, &is_font, &mut process);
    }

    pub fn make_absolute(&mut self) -> Result<()> {
        for file in &mut self.files {
            file.path = absolute(&file.path)?;
        }

        Ok(())
    }

    pub fn file_by_font_name(&self, name: &str) -> Option<&PathBuf> {
        if let Some(&file_idx) = self.map.get(name) {
            return Some(&self.files[file_idx].path);
        }

        // fallback:
        // match variable fonts, only by family name
        // assuming variable fonts provide any weight
        let (family, _) = parse_style(name);
        if let Some(&file_idx) = self.map.get(family) {
            let file = &self.files[file_idx];
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
