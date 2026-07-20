use std::path::PathBuf;

use clap::{Parser, Subcommand};

// default mode

/// Temporarily install fonts in (A)SSA subtitles
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Load font files
    Load {
        /// Font files or directories containing them
        source: Vec<PathBuf>,
        /// Recursively scan these directories
        #[arg(short, long = "recursive", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,
    },

    /// Load used fonts in (A)SSA subtitles
    LoadBy {
        /// Subtitle files or directories containing them
        source: Vec<PathBuf>,
        /// Recursively scan these directories
        #[arg(short, long = "recursive", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,
        /// Specify database file
        #[arg(short, long = "db")]
        db_path: Option<PathBuf>,
        /// Load fonts listed in ./fonts.txt
        #[arg(short, long)]
        list: bool,
    },

    /// Build index cache
    Index {
        /// Font files or directories containing them
        source: Vec<PathBuf>,
        /// Recursively scan these directories
        #[arg(short, long = "recursive", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,
        /// Specify database file
        #[arg(short, long = "db")]
        db_path: Option<PathBuf>,
        /// Do not translate paths to absolute
        #[arg(short = 'p', long = "portable")]
        portable: bool,
        /// Reset and rebuild the database
        #[arg(short = 'b', long)]
        reset: bool,
    },

    /// List used fonts in (A)SSA subtitles
    List {
        /// Subtitle files or directories containing them
        source: Vec<PathBuf>,
        /// Recursively scan these directories
        #[arg(short, long = "recursive", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,
        /// Treat fonts in database as installed
        #[arg(short, long = "db")]
        db_path: Option<Option<PathBuf>>,
        /// Export font list to ./fonts.txt
        #[arg(short = 'l', long = "list")]
        export_font_list: bool,
        /// Copy installed fonts to specified directory
        #[arg(short = 'x', long = "export", value_name = "TARGET")]
        export_font_files: Option<PathBuf>,
    },
}

/// fntldr FontLoader mode
#[derive(Parser)]
#[command(version)]
pub struct FontLoaderCli {
    /// TrueType / OpenType font files
    #[arg(value_name = "FONT_FILE")]
    pub files: Vec<PathBuf>,
}

/// fntldr FontLoaderSub mode
#[derive(Parser)]
#[command(version)]
pub struct FontLoaderSubCli {
    /// Directories containing (A)SSA subtitle files
    #[arg(value_name = "SUBTITLES_DIR")]
    pub dirs: Vec<PathBuf>,
}

/// fntldr ListAssFonts mode
#[derive(Parser)]
#[command(version)]
pub struct ListAssFontsCli {
    /// Directories containing (A)SSA subtitle files
    #[arg(value_name = "SUBTITLES_DIR")]
    pub dirs: Vec<PathBuf>,
}
