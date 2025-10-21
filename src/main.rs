mod app;
mod font;
mod ssa;
mod system;
mod utils;

use crate::app::*;

fn tldr() -> ! {
    println!("Made with curiosity by OV");
    std::process::exit(0);
}

fn main() {
    // `current_exe` follows symlink on linux
    let program_name = std::env::current_exe()
        .expect("Cannot get current executable")
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_ascii_lowercase();
    let program_name = program_name.strip_suffix(".exe").unwrap_or(&program_name);

    let result = match program_name {
        "friedegg" => tldr(),
        "fontloader" => fontloader_app(),
        "fontloadersub" => fontloadersub_app(),
        "listassfonts" => listassfonts_app(),
        _ => app(),
    };

    if let Err(error) = result {
        eprintln!("{}\n", error);
        eprint!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
        std::process::exit(1);
    }
}
