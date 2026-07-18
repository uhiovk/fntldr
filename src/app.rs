mod cli;
mod functions;

use anyhow::Result;
use clap::Parser;

use self::cli::*;
use self::functions::*;
use crate::utils::get_db_path;

pub fn app() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Load { source, recursive_dirs } => load(source, recursive_dirs),
        Commands::LoadBy { source, recursive_dirs, db_path, list } => {
            load_by(source, recursive_dirs, db_path, list)
        }
        Commands::Index { source, recursive_dirs, db_path, portable, reset } => {
            index(source, recursive_dirs, db_path, portable, reset)
        }
        Commands::List { source, recursive_dirs, db_path, export_font_list, export_font_files } => {
            list(source, recursive_dirs, db_path, export_font_list, export_font_files)
        }
    }
}

pub fn fontloader_app() -> Result<()> {
    let mut cli = FontLoaderCli::parse();
    if cli.files.is_empty() {
        cli.files.push(".".into());
    };
    load(cli.files, vec![])
}

pub fn fontloadersub_app() -> Result<()> {
    let cli = FontLoaderSubCli::parse();
    if !get_db_path(Some(".".as_ref())).is_file() {
        eprintln!("Database not found, start building...");
        index(vec![], vec![".".into()], Some(".".into()), false, true)?;
    }
    load_by(vec![], cli.dirs, Some(".".into()), false)
}

pub fn listassfonts_app() -> Result<()> {
    let cli = ListAssFontsCli::parse();
    list(vec![], cli.dirs, None, false, None)?;
    eprintln!("Press enter to exit");
    let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}
