use anyhow::Result;
use std::collections::HashSet;
use std::fs::copy;
use std::path::PathBuf;

use crate::font::FontProviders;
use crate::ssa::SsaFonts;
use crate::system::*;
use crate::utils::{
    get_cache_path, get_cache_path_fallback, get_font_list_path, is_font,
    walk_dir,
};

pub fn load(
    direct_dirs: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
) -> Result<()> {
    let mut all_files = Vec::new();

    for dir in direct_dirs {
        walk_dir(&dir, false, &is_font, &mut |path| all_files.push(path));
    }

    for dir in recursive_dirs {
        walk_dir(&dir, true, &is_font, &mut |path| all_files.push(path));
    }

    all_files.extend(files.into_iter().filter(|file| is_font(file)));

    if all_files.is_empty() {
        println!("Nothing to load");
        return Ok(());
    }

    let mut loader = get_loader()?;

    loader.load(&all_files)?;

    println!("Loaded {} files", all_files.len());
    wait();

    Ok(())
}

pub fn load_by(
    direct_dirs: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    cache_path: Option<PathBuf>,
    load_font_list: bool,
) -> Result<()> {
    let cache =
        FontProviders::load(&get_cache_path_fallback(cache_path.as_deref()))?;

    let mut ssa_fonts = if load_font_list {
        SsaFonts::load().unwrap_or_else(|_| {
            eprintln!("Cannot load \"fonts.txt\", ignoring");
            SsaFonts::new()
        })
    } else {
        SsaFonts::new()
    };

    for dir in direct_dirs {
        ssa_fonts.index(&dir, false);
    }

    for dir in recursive_dirs {
        ssa_fonts.index(&dir, true);
    }

    if ssa_fonts.inner().is_empty() {
        println!("Nothing to load");
        return Ok(());
    }

    let finder = get_finder()?;
    let mut loader = get_loader()?;

    let (names, files): (Vec<_>, HashSet<_>) = ssa_fonts
        .sorted()
        .into_iter()
        .filter(|name| get_installed_file(name, &finder).is_none())
        .filter_map(|name| {
            let opt = cache.file_by_font_name(&name);
            if opt.is_none() {
                println!("Font \"{}\" missing in index", name);
            }
            opt.map(|file| (name, file))
        })
        .unzip();

    if files.is_empty() {
        println!("Nothing to load");
        return Ok(());
    }

    loader.load(files)?;

    println!("\nLoaded fonts:\n");
    println!("{}", names.join("\n"));
    wait();

    Ok(())
}

pub fn index(
    direct_dirs: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    cache_path: Option<PathBuf>,
    is_absolute: bool,
) -> Result<()> {
    let (cache_is_specified, cache_path) =
        (cache_path.is_some(), get_cache_path(cache_path.as_deref()));

    let mut cache = if cache_is_specified && cache_path.is_file() {
        println!("Loading cache from \"{}\"", cache_path.display());
        FontProviders::load(&cache_path)?
    } else {
        println!("Creating new cache");
        FontProviders::new()
    };

    for dir in direct_dirs {
        cache.index(&dir, false);
    }

    for dir in recursive_dirs {
        cache.index(&dir, true);
    }

    if is_absolute {
        cache.make_absolute()?;
    }

    cache.save(&cache_path)?;
    println!("Saved cache to \"{}\"", cache_path.display());

    Ok(())
}

pub fn list(
    direct_dirs: Vec<PathBuf>,
    recursive_dirs: Vec<PathBuf>,
    cache_path: Option<Option<PathBuf>>,
    export_font_list: bool,
    export_fonts: Option<PathBuf>,
) -> Result<()> {
    const INSTALLED_INDICATOR: &str = "*";
    const IN_INDEX_INDICATOR: &str = "-";
    const NOT_INSTALLED_INDICATOR: &str = " ";

    let mut ssa_fonts = SsaFonts::new();

    for dir in direct_dirs {
        ssa_fonts.index(&dir, false);
    }

    for dir in recursive_dirs {
        ssa_fonts.index(&dir, true);
    }

    let finder = get_finder()?;
    let cache = match &cache_path {
        Some(path_opt) => {
            FontProviders::load(&get_cache_path(path_opt.as_deref()))?
        }
        None => FontProviders::new(),
    };

    let (do_export_fonts, export_fonts_path) = match export_fonts {
        Some(path) => {
            if path.is_dir() {
                (true, path)
            } else {
                eprintln!("Path is not a directory: \"{}\"", path.display());
                (false, PathBuf::new())
            }
        }
        None => (false, PathBuf::new()),
    };

    if cache_path.is_some() {
        println!(
            "{} for installed, {} for indexed in cache\n",
            INSTALLED_INDICATOR, IN_INDEX_INDICATOR
        );
    }

    for name in ssa_fonts.sorted() {
        let installed_file = get_installed_file(&name, &finder);
        let cached_file = cache.file_by_font_name(&name).map(|p| p.to_owned());

        let file;

        if let Some(path) = installed_file {
            println!("[{}] {}", INSTALLED_INDICATOR, name);
            file = Some(path);
        } else if let Some(path) = cached_file {
            println!("[{}] {}", IN_INDEX_INDICATOR, name);
            file = Some(path);
        } else {
            println!("[{}] {}", NOT_INSTALLED_INDICATOR, name);
            file = None;
        }

        if do_export_fonts && let Some(file) = file {
            let filename = file.file_name().unwrap();
            if copy(&file, export_fonts_path.join(filename)).is_err() {
                eprintln!(
                    "Error copying from \"{}\" to \"{}\"",
                    file.display(),
                    export_fonts_path.display()
                )
            }
        }
    }

    if export_font_list {
        ssa_fonts.save()?;
        println!("Exported font list to \"./fonts.txt\"");
    }

    Ok(())
}

fn get_installed_file(name: &str, finder: &impl FindFont) -> Option<PathBuf> {
    finder.get_font_file(name).unwrap_or_else(|_| {
        eprintln!(
            "Error checking installation state of \"{}\", \
            treating as not installed",
            name
        );

        None
    })
}

fn wait() {
    let (tx, rx) = std::sync::mpsc::channel::<()>();

    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    println!("\nPress Ctrl+C or close the window to unload fonts...");
    let _ = rx.recv();
}
