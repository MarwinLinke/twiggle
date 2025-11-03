use crate::screen::Screen;

use crossterm::style::Stylize;
use std::collections::BTreeMap;

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Sub,
}

pub struct View {
    screen: Screen,
    keybinds: String,
    current_mode: Mode,
}

impl View {
    pub fn new(screen: Screen, keybinds: String) -> Self {
        View {
            screen,
            keybinds,
            current_mode: Mode::Normal,
        }
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }

    pub fn mode(&self) -> Mode {
        self.current_mode
    }

    pub fn prepare_screen(&mut self) -> std::io::Result<()> {
        self.screen.hide_cursor()?;
        self.screen.move_up()
    }

    pub fn tidy_up_screen(&mut self) -> std::io::Result<()> {
        self.screen.clear_rest()?;
        self.screen.show_cursor()
    }

    pub fn print_screen(
        &mut self,
        current_dir: &Path,
        char_map: &BTreeMap<char, Vec<PathBuf>>,
        files: &[PathBuf],
        ambiguous_char: Option<char>,
        current_page: Option<usize>,
    ) -> std::io::Result<()> {
        let current_location = current_dir.to_string_lossy();

        let header = format!(" {} ", current_location).black().on_blue().bold();

        self.screen.write(header)?;
        self.screen.empty_line()?;

        match self.current_mode {
            Mode::Normal => self.print_normal(char_map),
            Mode::Sub => self.print_sub(
                char_map,
                ambiguous_char.expect("No ambiguous char specified"),
                current_page.expect("Current page is not set."),
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

        self.screen.write(file_info)?;
        self.screen.empty_line()?;
        self.screen.write("<Esc> to cancel | <Enter> to confirm")?;

        Ok(())
    }

    fn print_normal(&mut self, char_map: &BTreeMap<char, Vec<PathBuf>>) -> std::io::Result<()> {
        self.screen
            .write(" Directories ".black().on_dark_yellow().bold())?;

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
                self.screen.write(multiple)?;
            } else {
                self.screen.write(single)?;
            }
        }

        self.screen.empty_line()?;

        Ok(())
    }

    fn print_sub(
        &mut self,
        char_map: &BTreeMap<char, Vec<PathBuf>>,
        ambiguous_char: char,
        current_page: usize,
    ) -> std::io::Result<()> {
        self.screen.write(
            format!(" Select [{}] ", &ambiguous_char)
                .black()
                .on_dark_green()
                .bold(),
        )?;

        //eprintln!("AMBIGUOUS CHAR: {}", ambiguous_char);

        let dir_entries = char_map
            .get(&ambiguous_char)
            .expect("Char not found in map.");

        let page_size = 10;
        let start_idx = current_page * page_size;
        let end_idx = (start_idx + page_size).min(dir_entries.len());

        let current_slice: Vec<&PathBuf> =
            dir_entries.iter().skip(start_idx).take(page_size).collect();

        let other_dirs: Vec<&PathBuf> = dir_entries
            .iter()
            .take(start_idx) // before current page
            .chain(dir_entries.iter().skip(end_idx)) // after current page
            .collect();

        for (i, dir) in current_slice.iter().enumerate() {
            let prefix = match self.keybinds.chars().nth(i) {
                Some(c) => c.to_string(),
                None => String::from("..."),
            };

            let dir_str = format!("  {}", dir.file_name().unwrap().to_string_lossy()).dark_green();
            self.screen.write(format!("[{}] {}", prefix, dir_str))?;
        }

        // WARNING: Might actually break with exactly 10 directories
        let max_pages = dir_entries.len() / 10;
        if max_pages > 0 {
            let other_dirs_info = format!(
                "Other Directories: {}",
                other_dirs
                    .iter()
                    .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            self.screen.write(other_dirs_info)?;

            let page_info = format!(" Page [{}/{}] ", current_page + 1, max_pages + 1)
                .black()
                .on_dark_green()
                .bold();

            let navigation_info = "<f> page forward | <b> page backward";

            self.screen
                .write(format!("{} {}", page_info, navigation_info))?;
        }

        self.screen.empty_line()?;

        Ok(())
    }
}
