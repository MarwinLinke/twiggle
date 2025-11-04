mod fetcher;
mod raw_mode;
mod screen;
mod visualize;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use fetcher::{get_dirs_files, starts_with};
use raw_mode::RawModeGuard;
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
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let _raw_mode = RawModeGuard::new();
    let keybinds = "1234567890";
    let screen = Screen::new();

    let mut view: View = View::new(screen, keybinds.to_string(), !args.no_colors, args.icons);
    let mut prefix = String::from("");
    let mut current_page: Option<usize> = None;
    let mut needs_redraw = true;

    loop {
        let current_dir = std::env::current_dir()?;
        let (dirs, files) = get_dirs_files()?;

        if needs_redraw {
            disable_raw_mode()?;
            view.prepare_screen()?;
            view.print_screen(&current_dir, &dirs, &files, &prefix, current_page)?;
            view.tidy_up_screen()?;
            enable_raw_mode()?;
        }

        needs_redraw = false;

        if let Event::Key(e) = event::read()? {
            match e.code {
                KeyCode::Char(c) => {
                    if c == '~' {
                        let home_dir = dirs::home_dir().expect("Could not find home directory");
                        needs_redraw = true;
                        env::set_current_dir(&home_dir)?;
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
                                needs_redraw = true;
                                view.change_mode(Mode::Sub);
                                current_page = Some(0);
                                prefix = c.to_string();
                                continue;
                            }

                            needs_redraw = true;
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
                                        needs_redraw = true;
                                    }
                                    'b' => {
                                        current_page = current_page.map(|p| p.saturating_sub(1));
                                        needs_redraw = true;
                                    }
                                    'n' => {
                                        current_page = current_page.map(|p| p + 1);
                                        if current_page.unwrap_or(0) > max_page {
                                            current_page = Some(0);
                                        }
                                        needs_redraw = true;
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
                                    needs_redraw = true;
                                    continue;
                                }
                            };

                            let dict = match filtered_dirs.get(index) {
                                Some(dict) => dict,
                                None => continue,
                            };

                            needs_redraw = true;
                            env::set_current_dir(dict.file_name().unwrap())?;
                            view.change_mode(Mode::Normal);
                        }
                    }
                }
                KeyCode::Backspace => {
                    needs_redraw = true;
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
                    println!("{}", env::current_dir().unwrap().display());
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
