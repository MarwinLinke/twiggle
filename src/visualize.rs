use crate::dir_util::build_char_map;
use crate::dir_util::get_name;
use crate::history::PathHistory;
use crate::icons::icon_for_file;
use crate::mode::Mode;
use crate::screen::Screen;

use crossterm::style::Color;
use crossterm::style::Stylize;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::path::Path;
use std::path::PathBuf;

pub struct View {
    screen: Screen,
    keybinds: String,
    current_mode: Mode,
    debug_messages: Vec<String>,
    use_colors: bool,
    use_icons: bool,
    use_debug: bool,
    is_dirty: bool,
}

impl Drop for View {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.screen.show_cursor();
    }
}

impl View {
    pub fn new(
        screen: Screen,
        keybinds: String,
        use_colors: bool,
        use_icons: bool,
        use_debug: bool,
    ) -> Self {
        View {
            screen,
            keybinds,
            current_mode: Mode::Normal,
            debug_messages: Vec::new(),
            use_colors,
            use_icons,
            use_debug,
            is_dirty: true,
        }
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }

    pub fn debug_message(&mut self, message: String) {
        if self.use_debug {
            self.dirty();
            self.debug_messages.push(message);
        }
    }

    pub fn dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn prepare_screen(&mut self) -> std::io::Result<()> {
        if self.is_dirty {
            disable_raw_mode()?;
            self.screen.hide_cursor()?;
            self.screen.move_up()?;
        }
        Ok(())
    }

    pub fn clear_rest(&mut self) -> std::io::Result<()> {
        if self.is_dirty {
            self.debug_messages.clear();
            self.screen.clear_rest()?;
            self.screen.show_cursor()?;
            self.is_dirty = false;
            enable_raw_mode()?;
        }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> std::io::Result<()> {
        disable_raw_mode()?;
        self.screen.move_up()?;
        self.screen.clear_rest()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // I know it's bad
    pub fn display(
        &mut self,
        current_dir: &Path,
        dirs: &[PathBuf],
        files: &[PathBuf],
        prefix: &str,
        current_page: usize,
        history: &PathHistory,
        cursor_index: &Option<usize>,
    ) -> std::io::Result<()> {
        if self.is_dirty {
            self.print_screen(
                current_dir,
                dirs,
                files,
                prefix,
                current_page,
                history,
                cursor_index,
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // I know it's bad
    fn print_screen(
        &mut self,
        current_dir: &Path,
        dirs: &[PathBuf],
        files: &[PathBuf],
        prefix: &str,
        current_page: usize,
        history: &PathHistory,
        cursor_index: &Option<usize>,
    ) -> std::io::Result<()> {
        if !self.debug_messages.is_empty() {
            self.screen.write(" Debug ".black().on_cyan().bold())?;

            for message in &self.debug_messages {
                self.screen.write(message.clone().cyan())?;
            }

            self.screen.empty_line()?;
        }

        let path_str = display_path(current_dir);

        let blue = self.color_or_white(Color::Blue);
        let header = format!(" {} ", path_str).black().on(blue).bold();

        let mut history_str = format!("[{}/{}]", history.index + 1, history.buffer.len()).blue();

        if history.index + 1 == history.buffer.len() {
            history_str = String::from("").blue();
        }

        self.screen.write(format!("{} {}", header, history_str))?;
        self.screen.empty_line()?;

        match self.current_mode {
            Mode::Normal => self.print_normal(dirs, cursor_index),
            Mode::Select => {
                self.print_select(dirs, prefix, current_page, cursor_index.unwrap_or(0))
            }
        }?;

        // let file_str = files
        //     .iter()
        //     .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
        //     .collect::<Vec<String>>()
        //     .join(" ");

        let file_icons = files
            .iter()
            .map(|file| {
                let icon = if self.use_icons {
                    &format!("{} ", icon_for_file(file))[..]
                } else {
                    ""
                };
                let name = get_name(file);
                format!("{}{}", icon, name)
            })
            .collect::<Vec<String>>()
            .join("  ");

        let magenta = self.color_or_white(Color::Magenta);

        let files_header_str = if files.is_empty() {
            " No Files "
        } else {
            " Files "
        };

        let files_header = files_header_str.black().on(magenta).bold();
        let file_info = format!("{} {}", files_header, file_icons.with(magenta));

        self.screen.write(file_info)?;
        self.screen.empty_line()?;
        self.screen
            .write("<Esc> to cancel | <Enter> to change directory")?;

        Ok(())
    }

    fn print_normal(
        &mut self,
        dirs: &[PathBuf],
        cursor_index: &Option<usize>,
    ) -> std::io::Result<()> {
        //let dir_single_icon = if self.use_icons { "  " } else { "" };
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

        for (index, directories_with_char) in char_map.iter().enumerate() {
            let (char, directories) = directories_with_char;
            let is_multiple = directories.len() > 1;

            let dir_str = directories
                .iter()
                .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(" ");

            let mut char_disp = char.white();

            if let Some(i) = cursor_index
                && i == &index
            {
                char_disp = char_disp.on_white().black();
            }

            if is_multiple {
                self.screen.write(format!(
                    "[{}?] {}",
                    char_disp,
                    format!("{}{}", dir_multiple_icon, dir_str).white()
                ))?;
            } else {
                let directory = &directories[0];
                let icon: &str = if self.use_icons {
                    &format!("{}  ", icon_for_file(directory))[..]
                } else {
                    ""
                };

                self.screen.write(format!(
                    "[{}] {}",
                    char_disp,
                    format!("{}{}", icon, dir_str).with(yellow)
                ))?;
            }
        }

        self.screen.empty_line()?;

        Ok(())
    }

    fn print_select(
        &mut self,
        dirs: &[PathBuf],
        prefix: &str,
        current_page: usize,
        cursor_index: usize,
    ) -> std::io::Result<()> {
        //let dir_single_icon = if self.use_icons { "  " } else { "" };

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

        for (i, directory) in current_slice.iter().enumerate() {
            let mut number = match self.keybinds.chars().nth(i) {
                Some(c) => c.to_string(),
                None => String::from("..."),
            }
            .white();

            if cursor_index == i {
                number = number.on_white().black();
            }

            let icon: &str = if self.use_icons {
                &format!("{}  ", icon_for_file(directory))[..]
            } else {
                ""
            };

            let dir_str = &get_name(directory)[prefix.len()..];
            let prefix_str = format!("{}{}", icon, prefix.underlined()).with(green);
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
