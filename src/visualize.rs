use crate::screen::Screen;

use crossterm::style::Stylize;
use std::collections::BTreeMap;

use std::path::Path;
use std::path::PathBuf;

pub enum Mode {
    Normal,
    Sub,
}

pub fn print_screen(
    screen: &mut Screen,
    current_dir: &Path,
    char_map: &BTreeMap<char, Vec<PathBuf>>,
    files: &[PathBuf],
    mode: &Mode,
    ambiguous_char: Option<char>,
    keybinds: &str,
) -> std::io::Result<()> {
    let current_location = current_dir.to_string_lossy();

    let header = format!(" {} ", current_location).black().on_blue().bold();

    screen.write(header)?;
    screen.empty_line()?;

    match mode {
        Mode::Normal => print_normal(screen, char_map),
        Mode::Sub => print_sub(
            screen,
            char_map,
            ambiguous_char.expect("No ambiguous char specified"),
            keybinds,
        ),
    }?;

    let file_str = files
        .iter()
        .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let file_info = format!(
        "{} {}",
        " Files ".black().on_magenta().bold(),
        file_str.magenta()
    );

    screen.write(file_info)?;
    screen.empty_line()?;
    screen.write("<Esc> to cancel | <Enter> to confirm")?;

    Ok(())
}

fn print_normal(
    screen: &mut Screen,
    char_map: &BTreeMap<char, Vec<PathBuf>>,
) -> std::io::Result<()> {
    screen.write(" Directories ".black().on_dark_yellow().bold())?;

    for dir in char_map {
        let dir_str = dir
            .1
            .iter()
            .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let single = format!("[{}] {}", dir.0, format!("  {}", dir_str).dark_yellow());
        let multiple = format!("[{}?] {}", dir.0, format!("󰉓  {}", dir_str).white());

        if dir.1.len() > 1 {
            screen.write(multiple)?;
        } else {
            screen.write(single)?;
        }
    }

    screen.empty_line()?;

    Ok(())
}

fn print_sub(
    screen: &mut Screen,
    char_map: &BTreeMap<char, Vec<PathBuf>>,
    ambiguous_char: char,
    keybinds: &str,
) -> std::io::Result<()> {
    screen.write(
        format!(" Select [{}] ", &ambiguous_char)
            .black()
            .on_dark_green()
            .bold(),
    )?;

    //eprintln!("AMBIGUOUS CHAR: {}", ambiguous_char);

    let dir_entries = char_map
        .get(&ambiguous_char)
        .expect("Char not found in map.");

    for (i, dir) in dir_entries.iter().enumerate() {
        let prefix = match keybinds.chars().nth(i) {
            Some(c) => c.to_string(),
            None => String::from("..."),
        };

        let dir_str = format!("  {}", dir.file_name().unwrap().to_string_lossy()).dark_green();
        screen.write(format!("[{}] {}", prefix, dir_str))?;
    }

    screen.empty_line()?;

    Ok(())
}
