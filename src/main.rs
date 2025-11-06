mod dir_util;
mod icons;
mod screen;
mod visualize;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::disable_raw_mode;
use dir_util::{get_dirs_files, starts_with};
use screen::Screen;
use std::{env, io};
use visualize::{Mode, View};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    no_colors: bool,

    #[arg(short, long, default_value_t = false)]
    icons: bool,

    #[arg(long, default_value_t = false)]
    debug: bool,
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
    let mut prefix = String::from("");
    let mut current_page: Option<usize> = None;

    loop {
        let current_dir = std::env::current_dir()?;
        let (dirs, files) = get_dirs_files()?;

        view.prepare_screen()?;
        view.display(
            &current_dir,
            &dirs,
            &files,
            &prefix,
            current_page.unwrap_or_default(),
        )?;
        view.clear_screen()?;

        if let Event::Key(e) = event::read()? {
            view.debug_message(format!("Current char: {} {}", e.code, e.modifiers));
            match e.code {
                KeyCode::Char(c) => {
                    if c == '~' {
                        let home_dir = dirs::home_dir().expect("Could not find home directory");
                        env::set_current_dir(&home_dir)?;
                        view.dirty();
                        view.change_mode(Mode::Normal);
                        continue;
                    }

                    match view.mode() {
                        Mode::Normal => {
                            prefix = String::from("");
                            let filtered_dirs = starts_with(&dirs, &c.to_string());

                            if filtered_dirs.is_empty() {
                                continue;
                            }

                            if filtered_dirs.len() > 1 {
                                view.dirty();
                                view.change_mode(Mode::Sub);
                                current_page = Some(0);
                                prefix = c.to_string();
                                continue;
                            }

                            view.dirty();
                            env::set_current_dir(filtered_dirs[0].file_name().unwrap())?;
                        }
                        Mode::Sub => {
                            let filtered_dirs = starts_with(&dirs, &prefix);

                            let n = filtered_dirs.len();
                            let max_page = if n == 0 { 0 } else { (n - 1) / 10 };

                            if e.modifiers.contains(KeyModifiers::CONTROL) {
                                match c {
                                    'f' => {
                                        current_page = current_page.map(|p| (p + 1).min(max_page));
                                        view.dirty();
                                    }
                                    'b' => {
                                        current_page = current_page.map(|p| p.saturating_sub(1));
                                        view.dirty();
                                    }
                                    'n' => {
                                        current_page = current_page.map(|p| p + 1);
                                        if current_page.unwrap_or(0) > max_page {
                                            current_page = Some(0);
                                        }
                                        view.dirty();
                                    }
                                    _ => (),
                                }
                                continue;
                            }

                            let index = match keybinds.find(c) {
                                Some(i) => i + current_page.unwrap_or_default() * 10,
                                None => {
                                    prefix.push(c);
                                    current_page = Some(0);
                                    view.dirty();
                                    continue;
                                }
                            };

                            let dict = match filtered_dirs.get(index) {
                                Some(dict) => dict,
                                None => continue,
                            };

                            env::set_current_dir(dict.file_name().unwrap())?;
                            view.dirty();
                            view.change_mode(Mode::Normal);
                        }
                    }
                }
                KeyCode::Backspace => {
                    view.dirty();
                    if matches!(view.mode(), Mode::Sub) {
                        if prefix.len() > 1 {
                            prefix.pop();
                            current_page = Some(0);
                        } else {
                            view.change_mode(Mode::Normal);
                        }
                    } else {
                        env::set_current_dir("..")?;
                    }
                }
                KeyCode::Esc => {
                    println!(".");
                    break;
                }
                KeyCode::Enter => {
                    disable_raw_mode()?;
                    println!("{}", env::current_dir().unwrap().display());
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
