use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

// default mode

/// fntldr by uHioVK
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
        /// Directories to be scanned
        #[arg(short, long = "dir", value_name = "DIRECTORY")]
        direct_dirs: Vec<PathBuf>,

        /// Directories to be recursive scanned
        #[arg(short, long = "recurse", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,

        /// Font files
        #[arg(value_name = "FONT_FILE")]
        files: Vec<PathBuf>,
    },

    /// Load used fonts in (A)SSA subtitles
    LoadBy {
        /// Directories to be scanned
        #[arg(short, long = "dir", value_name = "DIRECTORY")]
        direct_dirs: Vec<PathBuf>,

        /// Directories to be recursive scanned
        #[arg(short, long = "recurse", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,

        /// Manually specify cache file
        #[arg(short, long)]
        cache: Option<PathBuf>,

        /// Load fonts listed in ./fonts.txt
        #[arg(short = 'l', long = "font-list")]
        load_font_list: bool,
    },

    /// Build index cache
    Index {
        /// Directories to be scanned
        #[arg(short, long = "dir", value_name = "DIRECTORY")]
        direct_dirs: Vec<PathBuf>,

        /// Directories to be recursive scanned
        #[arg(short, long = "recurse", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,

        /// Manually specify cache file to save to
        #[arg(short, long)]
        cache: Option<PathBuf>,

        /// Avoid translate saved paths into absolute
        #[arg(short = 'p', long = "portable", action = ArgAction::SetFalse)]
        is_absolute: bool,
    },

    /// List used fonts in (A)SSA subtitles
    List {
        /// Directories to be scanned
        #[arg(short, long = "dir", value_name = "DIRECTORY")]
        direct_dirs: Vec<PathBuf>,

        /// Directories to be recursive scanned
        #[arg(short, long = "recurse", value_name = "DIRECTORY")]
        recursive_dirs: Vec<PathBuf>,

        /// Mark fonts listed in cache as installed,
        /// use default cache if not specified
        #[arg(short, long)]
        cache: Option<Option<PathBuf>>,

        /// Export font list to ./fonts.txt
        #[arg(short = 'l', long = "font-list")]
        export_font_list: bool,

        /// Export installed fonts to directory
        #[arg(short = 'x', long = "export", value_name = "DIRECTORY")]
        export_fonts: Option<PathBuf>,
    },
}

// FontLoader mode

#[derive(Parser)]
#[command(version)]
pub struct FontLoaderCli {
    /// TrueType / OpenType font files
    #[arg(value_name = "FONT_FILE")]
    pub files: Vec<PathBuf>,
}

// FontLoaderSub mode

#[derive(Parser)]
#[command(version)]
pub struct FontLoaderSubCli {
    /// Directories containing (A)SSA subtitle files
    #[arg(value_name = "SUBTITLES_DIR")]
    pub dirs: Vec<PathBuf>,
}

// ListAssFonts mode

#[derive(Parser)]
#[command(version)]
pub struct ListAssFontsCli {
    /// Directories containing (A)SSA subtitle files
    #[arg(value_name = "SUBTITLES_DIR")]
    pub dirs: Vec<PathBuf>,
}
