mod app;
mod font;
mod ssa;
mod system;
mod utils;

use std::path::PathBuf;
use std::{io, process};

use crate::app::*;

fn tldr() -> ! {
    eprintln!("Made with curiosity by OV");
    eprintln!("Thank you for supporting!");
    std::process::exit(0);
}

fn main() {
    let invocation_name = PathBuf::from(std::env::args().next().unwrap());
    let program_name = invocation_name.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let result = match program_name {
        "friedegg" => tldr(),
        "fontloader" => fontloader_app(),
        "fontloadersub" => fontloadersub_app(),
        "listassfonts" => listassfonts_app(),
        _ => app(),
    };

    if let Err(error) = result {
        eprintln!("{}\n", error);
        eprintln!("Press enter to exit...");
        let _ = io::stdin().read_line(&mut String::new());
        process::exit(1);
    }
}
