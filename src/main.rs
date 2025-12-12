mod dir_util;
mod history;
mod icons;
mod mode;
mod screen;
mod visualize;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::disable_raw_mode;
use dir_util::{get_dirs_files, starts_with};
use history::PathHistory;
use screen::Screen;
use std::path::PathBuf;
use std::{env, io};
use visualize::View;

use crate::dir_util::filter_hidden;
use crate::mode::Mode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    no_colors: bool,

    #[arg(short, long, default_value_t = false)]
    icons: bool,

    #[arg(long, default_value_t = false)]
    debug: bool,

    #[arg(long, default_value_t = false)]
    hide: bool,

    #[arg(long, default_value_t = false)]
    clear: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let keybinds = "1234567890";

    let screen = Screen::new();
    let mut view: View = View::new(
        screen,
        keybinds.to_string(),
        !args.no_colors,
        args.icons,
        args.debug,
    );

    input_loop(&mut view, keybinds, !args.hide)?;

    if args.clear {
        view.clear_screen()?;
    }

    Ok(())
}

fn input_loop(view: &mut View, keybinds: &str, show_hidden_default: bool) -> io::Result<()> {
    let mut prefix = String::from("");
    let mut current_page: Option<usize> = None;
    let mut show_hidden = show_hidden_default;
    let mut mode = Mode::Normal;
    let mut is_dirty;
    let mut has_terminated;
    let mut history = PathHistory::new();
    history.push(env::current_dir()?);

    loop {
        let current_dir = std::env::current_dir()?;
        let (mut dirs, mut files) = get_dirs_files()?;

        if !show_hidden {
            dirs = filter_hidden(&dirs);
            files = filter_hidden(&files);
        }

        view.debug_message(format!("Show hidden files: {}", show_hidden));
        view.debug_message(format!("Index: {}", history.index));
        view.debug_message(format!("History Length: {}", history.buffer.len()));

        view.change_mode(mode);
        view.prepare_screen()?;
        view.display(
            &current_dir,
            &dirs,
            &files,
            &prefix,
            current_page.unwrap_or_default(),
        )?;
        view.clear_rest()?;

        if let Event::Key(e) = event::read()? {
            // view.debug_message(format!("Current char: {} {}", e.code, e.modifiers));
            match mode {
                Mode::Normal => {
                    (is_dirty, has_terminated) = handle_normal_mode(
                        e,
                        &mut prefix,
                        &mut show_hidden,
                        &mut mode,
                        &mut current_page,
                        &dirs,
                        &mut history,
                    )?;
                }
                Mode::Select => {
                    (is_dirty, has_terminated) = handle_select_mode(
                        e,
                        &mut prefix,
                        &mut show_hidden,
                        &mut mode,
                        &mut current_page,
                        keybinds,
                        &dirs,
                        &mut history,
                    )?;
                }
            }

            if is_dirty {
                view.dirty();
            }

            if has_terminated {
                break;
            }
        }
    }
    Ok(())
}

fn handle_normal_mode(
    e: KeyEvent,
    prefix: &mut String,
    show_hidden: &mut bool,
    mode: &mut Mode,
    current_page: &mut Option<usize>,
    dirs: &[PathBuf],
    history: &mut PathHistory,
) -> io::Result<(bool, bool)> {
    match e.code {
        KeyCode::Char(c) => {
            if check_hidden_key(e, c, show_hidden) || check_home_key(c, mode, history)? {
                return Ok((true, false));
            };

            *prefix = String::from("");
            let filtered_dirs = starts_with(dirs, &c.to_string());

            if filtered_dirs.is_empty() {
                return Ok((false, false));
            }

            if filtered_dirs.len() > 1 {
                *mode = Mode::Select;
                *current_page = Some(0);
                *prefix = c.to_string();
                return Ok((true, false));
            }

            env::set_current_dir(filtered_dirs[0].file_name().unwrap())?;
            history.push(env::current_dir()?);

            Ok((true, false))
        }
        KeyCode::Backspace => {
            env::set_current_dir("..")?;
            history.push(env::current_dir()?);

            Ok((true, false))
        }
        KeyCode::Esc => {
            println!(".");
            Ok((false, true))
        }
        KeyCode::Enter => {
            disable_raw_mode()?;
            println!("{}", env::current_dir().unwrap().display());
            Ok((false, true))
        }
        KeyCode::Up => {
            let path = history.go_up();

            match path {
                Some(p) => {
                    env::set_current_dir(p)?;
                    Ok((true, false))
                }
                None => Ok((false, false)),
            }
        }
        KeyCode::Down => {
            let path = history.go_down();

            match path {
                Some(p) => {
                    env::set_current_dir(p)?;
                    Ok((true, false))
                }
                None => Ok((false, false)),
            }
        }
        _ => Ok((false, false)),
    }
}

#[allow(clippy::too_many_arguments)] // I know it's bad
fn handle_select_mode(
    e: KeyEvent,
    prefix: &mut String,
    show_hidden: &mut bool,
    mode: &mut Mode,
    current_page: &mut Option<usize>,
    keybinds: &str,
    dirs: &[PathBuf],
    history: &mut PathHistory,
) -> io::Result<(bool, bool)> {
    match e.code {
        KeyCode::Char(c) => {
            if check_hidden_key(e, c, show_hidden) || check_home_key(c, mode, history)? {
                return Ok((true, false));
            };

            let filtered_dirs = starts_with(dirs, prefix);

            let n = filtered_dirs.len();
            let max_page = if n == 0 { 0 } else { (n - 1) / 10 };

            if e.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'f' => {
                        *current_page = current_page.map(|p| (p + 1).min(max_page));
                    }
                    'b' => {
                        *current_page = current_page.map(|p| p.saturating_sub(1));
                    }
                    'n' => {
                        *current_page = current_page.map(|p| p + 1);
                        if current_page.unwrap_or(0) > max_page {
                            *current_page = Some(0);
                        }
                    }
                    _ => return Ok((false, false)),
                }
                return Ok((true, false));
            }

            let index = match keybinds.find(c) {
                Some(i) => i + current_page.unwrap_or_default() * 10,
                None => {
                    prefix.push(c);
                    *current_page = Some(0);
                    return Ok((true, false));
                }
            };

            let dict = match filtered_dirs.get(index) {
                Some(dict) => dict,
                None => return Ok((false, false)),
            };

            env::set_current_dir(dict.file_name().unwrap())?;
            history.push(env::current_dir()?);
            *mode = Mode::Normal;
            Ok((true, false))
        }
        KeyCode::Backspace => {
            if prefix.len() > 1 {
                prefix.pop();
                *current_page = Some(0);
            } else {
                *mode = Mode::Normal;
            }
            Ok((true, false))
        }
        KeyCode::Esc => {
            prefix.clear();
            *current_page = Some(0);
            *mode = Mode::Normal;
            Ok((true, false))
        }
        KeyCode::Enter => {
            let filtered_dirs = starts_with(dirs, prefix);
            let dict = match filtered_dirs.first() {
                Some(dict) => dict,
                None => return Ok((false, false)),
            };

            env::set_current_dir(dict.file_name().unwrap())?;
            history.push(env::current_dir()?);
            *mode = Mode::Normal;
            Ok((true, false))
        }
        _ => Ok((false, false)),
    }
}

fn check_hidden_key(e: KeyEvent, c: char, show_hidden: &mut bool) -> bool {
    if e.modifiers.contains(KeyModifiers::CONTROL) && c == 's' {
        *show_hidden = !*show_hidden;
        return true;
    }
    false
}

fn check_home_key(c: char, mode: &mut Mode, history: &mut PathHistory) -> io::Result<bool> {
    if c == '~' {
        let home_dir = dirs::home_dir().expect("Could not find home directory.");
        env::set_current_dir(&home_dir)?;
        history.push(env::current_dir()?);
        *mode = Mode::Normal;
        return Ok(true);
    }
    Ok(false)
}
