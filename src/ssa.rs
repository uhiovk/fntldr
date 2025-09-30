use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;

// this crate is very probably using tons of LLM generated code
// I definitely don't like that, but at least it has fairly nice API
// and there is not a single crate else that follows basic SSA specs
use ass_core::{Script, Section, parser::SectionType};

use crate::utils::{is_ssa, walk_dir};

pub struct SsaFonts(HashSet<String>);

impl SsaFonts {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn inner(&self) -> &HashSet<String> {
        &self.0
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = read_to_string(path).with_context(|| {
            format!("Error reading file \"{}\"", path.display())
        })?;

        Ok(content.parse().unwrap())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            create_dir_all(dir).with_context(|| {
                format!("Error creating directory \"{}\"", path.display())
            })?;
        }

        write(path, self.to_string()).with_context(|| {
            format!("Error writing file \"{}\"", path.display())
        })?;

        Ok(())
    }

    pub fn index(&mut self, path: &Path, is_recursive: bool) {
        let mut process =
            |path: PathBuf| self.0.extend(Self::get_ssa_fonts(&path));

        walk_dir(path, is_recursive, &is_ssa, &mut process)
    }

    pub fn sorted(&self) -> Vec<String> {
        let mut vec: Vec<_> = self.0.iter().cloned().collect();
        vec.sort_unstable();
        vec
    }

    fn get_ssa_fonts(path: &Path) -> HashSet<String> {
        fn strip_prefix(s: &str) -> String {
            s.strip_prefix('@').unwrap_or(s).to_string()
        }

        let Ok(content) = read_to_string(path) else {
            eprintln!("Error reading file \"{}\"", path.display());
            return HashSet::new();
        };

        let Ok(sub) = Script::parse(&content) else {
            eprintln!("Error parsing (A)SSA file \"{}\"", path.display());
            return HashSet::new();
        };

        let Some(Section::Styles(styles)) =
            sub.find_section(SectionType::Styles)
        else {
            eprintln!(
                "The script does not contain styles section: \"{}\"",
                path.display()
            );
            return HashSet::new();
        };

        let Some(Section::Events(events)) =
            sub.find_section(SectionType::Events)
        else {
            eprintln!(
                "The script does not contain events section: \"{}\"",
                path.display()
            );
            return HashSet::new();
        };

        let mut fonts = HashSet::new();
        let mut used_styles = HashSet::new();

        for dialogue in events.iter().filter(|event| event.is_dialogue()) {
            used_styles.insert(dialogue.style.to_string());

            // add all inline font overrides in the dialogue
            fonts.extend(
                FONT_OVRD_REGEX.captures_iter(dialogue.text).filter_map(
                    |cap| cap.get(1).map(|m| strip_prefix(m.as_str())),
                ),
            );
        }

        // extract fonts from used styles
        fonts.extend(styles.iter().filter_map(|style| {
            if used_styles.contains(style.name) {
                Some(strip_prefix(style.fontname))
            } else {
                None
            }
        }));

        fonts
    }
}

// simple one entry per line format
impl Display for SsaFonts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for str in self.sorted() {
            writeln!(f, "{str}")?;
        }

        Ok(())
    }
}

impl FromStr for SsaFonts {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.lines().map(String::from).collect()))
    }
}

// in SSA, "{\fnFont Name}" specifies a font override for following text
// multiple style overrides may be specified in a single pair of "{}"
// we only match the last specified font name in each "{}" as it would override previous ones
// test it out with "Hello, {\fnFoo Font\fs42\fnBar Font}Rust {\fnrustc\fs10\fncargo\b1}World!"
// it will capture "Bar Font" and "cargo"
static FONT_OVRD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}]*\\fn([^}\\]+).*?}").unwrap());
