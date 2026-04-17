use std::collections::HashSet;
use std::fs::copy;
use std::path::PathBuf;

use anyhow::Result;

use crate::font::FontProviders;
use crate::ssa::SsaFonts;
use crate::system::{FindFont, Finder, LoadFont, Loader};
use crate::utils::{get_db_path, is_font, walk_dir};

pub fn load(source: Vec<PathBuf>, recursive_dirs: Vec<PathBuf>) -> Result<()> {
    let mut all_files = Vec::new();

    for path in source {
        if path.is_dir() {
            walk_dir(&path, false, &is_font, &mut |path| all_files.push(path));
        } else {
            all_files.push(path);
        }
    }

    for dir in recursive_dirs {
        walk_dir(&dir, true, &is_font, &mut |path| all_files.push(path));
    }

    if all_files.is_empty() {
        eprintln!("Nothing to load");
        return Ok(());
    }

    let mut loader = Loader::new()?;
    loader.load(&all_files)?;

    eprintln!("Successfully loaded {} files", all_files.len());
    wait();

    Ok(())
}

pub fn load_by(
    source: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    db_path: Option<PathBuf>,
    load_font_list: bool,
) -> Result<()> {
    let mut ssa_fonts = if load_font_list {
        SsaFonts::load("fonts.txt".as_ref()).unwrap_or_else(|_| {
            eprintln!("Cannot read \"fonts.txt\", ignoring");
            SsaFonts::new()
        })
    } else {
        SsaFonts::new()
    };

    for path in source {
        if path.is_dir() {
            ssa_fonts.scan_dir(&path, false);
        } else {
            ssa_fonts.add_file(&path);
        }
    }

    for dir in recursive_dirs {
        ssa_fonts.scan_dir(&dir, true);
    }

    if ssa_fonts.as_inner().is_empty() {
        eprintln!("Nothing to load");
        return Ok(());
    }

    let finder = Finder::new()?;
    let db = FontProviders::load(&get_db_path(db_path.as_deref()))?;
    let (names, files): (Vec<_>, HashSet<_>) = ssa_fonts
        .sorted()
        .into_iter()
        .filter(|name| matches!(finder.get_font_file(name), Ok(None)))
        .filter_map(|name| {
            let opt = db.get_file(&name);
            if opt.is_none() {
                eprintln!("Font \"{}\" missing in index", name);
            }
            opt.map(|file| (name, file))
        })
        .unzip();

    if files.is_empty() {
        eprintln!("Nothing to load");
        return Ok(());
    }

    let mut loader = Loader::new()?;
    loader.load(files)?;

    eprintln!("\nLoaded fonts:\n");
    for name in names {
        eprintln!("{name}");
    }
    wait();

    Ok(())
}

pub fn index(
    source: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    db_path: Option<PathBuf>,
    portable: bool,
    reset: bool,
) -> Result<()> {
    let (cache_is_specified, db_path) = (db_path.is_some(), get_db_path(db_path.as_deref()));

    let mut db = if !reset && cache_is_specified && db_path.is_file() {
        FontProviders::load(&db_path)?
    } else {
        FontProviders::new()
    };

    for path in source {
        if path.is_dir() {
            db.scan_dir(&path, false);
        } else {
            db.add_file(path);
        }
    }

    for dir in recursive_dirs {
        db.scan_dir(&dir, true);
    }

    if !portable {
        db.make_absolute()?;
    }

    db.save(&db_path)?;

    Ok(())
}

pub fn list(
    source: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    db_path: Option<Option<PathBuf>>,
    export_font_list: bool,
    export_font_files: Option<PathBuf>,
) -> Result<()> {
    const INSTALLED_INDICATOR: &str = "*";
    const IN_INDEX_INDICATOR: &str = "-";
    const NOT_INSTALLED_INDICATOR: &str = " ";

    #[cfg(target_os = "windows")]
    if export_font_files.is_some() {
        unimplemented!("Exporting fonts on Windows is not yet implemeted");
    }

    let mut ssa_fonts = SsaFonts::new();

    for path in source {
        if path.is_file() {
            ssa_fonts.add_file(&path);
        } else {
            ssa_fonts.scan_dir(&path, false);
        }
    }

    for dir in recursive_dirs {
        ssa_fonts.scan_dir(&dir, true);
    }

    let export_font_path = export_font_files.and_then(|path| {
        if path.is_dir() {
            Some(path)
        } else {
            eprintln!("Path is not a directory: \"{}\"", path.display());
            None
        }
    });

    if db_path.is_some() {
        eprintln!(
            "{} for installed, {} for indexed in cache\n",
            INSTALLED_INDICATOR, IN_INDEX_INDICATOR
        );
    }

    let db = match &db_path {
        Some(path_opt) => Some(FontProviders::load(&get_db_path(path_opt.as_deref()))?),
        None => None,
    };

    let finder = Finder::new()?;
    for name in ssa_fonts.sorted() {
        let file = if let Some(path) = finder.get_font_file(&name)? {
            eprintln!("[{}] {}", INSTALLED_INDICATOR, name);
            Some(path)
        } else if let Some(db) = &db
            && let Some(path) = db.get_file(&name)
        {
            eprintln!("[{}] {}", IN_INDEX_INDICATOR, name);
            Some(path.to_owned())
        } else {
            eprintln!("[{}] {}", NOT_INSTALLED_INDICATOR, name);
            None
        };

        if let Some(export_path) = &export_font_path
            && let Some(file) = file
        {
            let filename = file.file_name().unwrap();
            if copy(&file, export_path.join(filename)).is_err() {
                eprintln!(
                    "Cannot copy from \"{}\" to \"{}\"",
                    file.display(),
                    export_path.display()
                )
            }
        }
    }

    if export_font_list {
        ssa_fonts.save("fonts.txt".as_ref())?;
        eprintln!("Exported font list to \"./fonts.txt\"");
    }

    Ok(())
}

fn wait() {
    let (tx, rx) = std::sync::mpsc::channel::<()>();

    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    eprintln!("\nPress Ctrl+C to unload fonts...");
    let _ = rx.recv();
}
