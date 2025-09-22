mod app;
mod font;
mod ssa;
mod system;

use crate::app::*;

fn tldr() -> ! {
    println!("Made with curiosity by OV");
    std::process::exit(0);
}

fn main() {
    let program_name = std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_ascii_lowercase()
        .to_string_lossy()
        .into_owned();
    let program_name =
        program_name.strip_suffix(".exe").unwrap_or(&program_name);

    let result = match program_name {
        "friedegg" => tldr(),
        "fontloader" => fontloader_app(),
        "fontloadersub" => fontloadersub_app(),
        "listassfonts" => listassfonts_app(),
        _ => app(),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        eprintln!();
        eprint!("Press enter to exit...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
        std::process::exit(1);
    }
}
