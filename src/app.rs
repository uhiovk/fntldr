mod cli;
mod functions;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use self::cli::*;
use self::functions::*;
use crate::utils::get_cache_path;

pub fn app() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Load { direct_dirs, recursive_dirs, files } => {
            load(direct_dirs, recursive_dirs, files)
        }

        Commands::LoadBy { direct_dirs, recursive_dirs, cache, load_font_list } => {
            load_by(direct_dirs, recursive_dirs, cache, load_font_list)
        }

        Commands::Index { direct_dirs, recursive_dirs, cache, is_absolute, rebuild } => {
            index(direct_dirs, recursive_dirs, cache, is_absolute, rebuild)
        }

        Commands::List {
            direct_dirs,
            recursive_dirs,
            cache,
            export_font_list,
            export_fonts_path,
        } => list(direct_dirs, recursive_dirs, cache, export_font_list, export_fonts_path),

        Commands::Clear { cache } => clear(cache),
    }
}

pub fn fontloader_app() -> Result<()> {
    let cli = FontLoaderCli::parse();
    let direct_dirs = if cli.files.is_empty() { vec![PathBuf::from(".")] } else { vec![] };
    load(direct_dirs, vec![], cli.files)
}

pub fn fontloadersub_app() -> Result<()> {
    let cli = FontLoaderSubCli::parse();
    if !get_cache_path(Some(&PathBuf::from("."))).is_file() {
        eprintln!("Cache not found, building...");
        index(vec![], vec![PathBuf::from(".")], Some(PathBuf::from(".")), false, false)?;
    }
    load_by(vec![], cli.dirs, Some(PathBuf::from(".")), false)
}

pub fn listassfonts_app() -> Result<()> {
    let cli = ListAssFontsCli::parse();
    list(vec![], cli.dirs, None, false, None)?;
    println!("Press enter to exit");
    let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}
