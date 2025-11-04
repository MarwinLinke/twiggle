use crate::screen::Screen;

use crate::fetcher::build_char_map;
use crossterm::style::Color;
use crossterm::style::Stylize;

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
    use_colors: bool,
    use_icons: bool,
}

impl View {
    pub fn new(screen: Screen, keybinds: String, use_colors: bool, use_icons: bool) -> Self {
        View {
            screen,
            keybinds,
            current_mode: Mode::Normal,
            use_colors,
            use_icons,
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
        dirs: &[PathBuf],
        files: &[PathBuf],
        prefix: &str,
        current_page: Option<usize>,
    ) -> std::io::Result<()> {
        let path_str = display_path(current_dir);

        let blue = self.color_or_white(Color::Blue);
        let header = format!(" {} ", path_str).black().on(blue).bold();

        self.screen.write(header)?;
        self.screen.empty_line()?;

        match self.current_mode {
            Mode::Normal => self.print_normal(dirs),
            Mode::Sub => self.print_sub(
                dirs,
                prefix,
                current_page.expect("Current page is not set."),
            ),
        }?;

        let file_str = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let magenta = self.color_or_white(Color::Magenta);
        let files_header_str = if files.is_empty() {
            " No Files "
        } else {
            " Files "
        };
        let files_header = files_header_str.black().on(magenta).bold();

        let file_info = format!("{} {}", files_header, file_str.with(magenta));

        self.screen.write(file_info)?;
        self.screen.empty_line()?;
        self.screen
            .write("<Esc> to cancel | <Enter> to change directory")?;

        Ok(())
    }

    fn print_normal(&mut self, dirs: &[PathBuf]) -> std::io::Result<()> {
        let dir_single_icon = if self.use_icons { "  " } else { "" };
        let dir_multiple_icon = if self.use_icons { "󰉓  " } else { "" };

        let yellow = self.color_or_white(Color::DarkYellow);
        let dirs_header_str = if dirs.is_empty() {
            " No Directories "
        } else {
            " Directories "
        };
        self.screen
            .write(dirs_header_str.black().on(yellow).bold())?;

        let char_map = build_char_map(dirs);

        for dir in char_map {
            let dir_str = dir
                .1
                .iter()
                .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(" ");

            let single = format!(
                "[{}] {}",
                dir.0,
                format!("{}{}", dir_single_icon, dir_str).with(yellow)
            );
            let multiple = format!(
                "[{}?] {}",
                dir.0,
                format!("{}{}", dir_multiple_icon, dir_str).white()
            );

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
        dirs: &[PathBuf],
        prefix: &str,
        current_page: usize,
    ) -> std::io::Result<()> {
        let dir_single_icon = if self.use_icons { "  " } else { "" };

        let green = self.color_or_white(Color::DarkGreen);
        self.screen
            .write(format!(" Select [{}] ", &prefix).black().on(green).bold())?;

        let filtered_dirs: Vec<PathBuf> = dirs
            .iter()
            .filter(|dir| {
                dir.file_name()
                    .map(|os_str| os_str.to_string_lossy().starts_with(prefix))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        let page_size = 10;
        let start_idx = current_page * page_size;
        let end_idx = (start_idx + page_size).min(filtered_dirs.len());

        let current_slice = &filtered_dirs[start_idx..end_idx];
        let other_dirs = [&filtered_dirs[..start_idx], &filtered_dirs[end_idx..]].concat();

        for (i, dir) in current_slice.iter().enumerate() {
            let number = match self.keybinds.chars().nth(i) {
                Some(c) => c.to_string(),
                None => String::from("..."),
            };

            let dir_str = &dir.file_name().unwrap().to_string_lossy().to_string()[prefix.len()..];
            let prefix_str = format!("{}{}", dir_single_icon, prefix.underlined()).with(green);
            self.screen.write(format!(
                "[{}] {}{}",
                number,
                prefix_str,
                dir_str.with(green)
            ))?;
        }

        let n = filtered_dirs.len();
        let max_pages = if n == 0 { 0 } else { (n - 1) / 10 };

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
                .on(green)
                .bold();

            let navigation_info = "<C-f> page forward | <C-b> page backward";

            self.screen
                .write(format!("{} {}", page_info, navigation_info))?;
        }

        self.screen.empty_line()?;

        Ok(())
    }

    fn color_or_white(&self, color: Color) -> Color {
        if self.use_colors { color } else { Color::White }
    }

    //fn color_or_black(&self, color: Color) -> Color {
    //    if self.use_colors { color } else { Color::Black }
    //}
}

fn display_path(path: &Path) -> String {
    dirs::home_dir()
        .and_then(|home| path.strip_prefix(&home).ok().map(|p| p.to_owned()))
        .map(|stripped| {
            if stripped.as_os_str().is_empty() {
                "~".to_string() // just "~" if it's the home directory
            } else {
                format!("~/{}", stripped.display())
            }
        })
        .unwrap_or_else(|| path.display().to_string())
}
