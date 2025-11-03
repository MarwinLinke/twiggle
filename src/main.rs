mod fetcher;
mod raw_mode;
mod screen;
mod visualize;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use fetcher::get_dirs;
use raw_mode::RawModeGuard;
use screen::Screen;
use std::{env, io};
use visualize::{Mode, View};

fn main() -> io::Result<()> {
    let _raw_mode = RawModeGuard::new();

    let keybinds = "1234567890";
    let screen = Screen::new();

    let mut view: View = View::new(screen, keybinds.to_string());

    let mut ambiguous_char: Option<char> = None;
    let mut current_page: Option<usize> = None;

    let mut needs_redraw = true;

    loop {
        let current_dir = std::env::current_dir()?;
        let (char_map, files) = get_dirs()?;

        if needs_redraw {
            disable_raw_mode()?;
            //eprintln!("{:?}", char_map);
            view.prepare_screen()?;
            view.print_screen(
                &current_dir,
                &char_map,
                &files,
                ambiguous_char,
                current_page,
            )?;
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
                            let dict_entries = match char_map.get(&c) {
                                Some(entry) => entry,
                                None => continue, // early exit if not found
                            };

                            if dict_entries.len() > 1 {
                                needs_redraw = true;
                                view.change_mode(Mode::Sub);
                                current_page = Some(0);
                                ambiguous_char = Some(c);
                                continue;
                            }

                            needs_redraw = true;
                            env::set_current_dir(dict_entries[0].file_name().unwrap())?;
                        }
                        Mode::Sub => {
                            let dicts = char_map.get(&ambiguous_char.unwrap()).unwrap();

                            let max_page = dicts.len() / 10;

                            match c {
                                'n' | 'f' => {
                                    current_page = current_page.map(|p| (p + 1).min(max_page));
                                    needs_redraw = true;
                                }
                                'b' => {
                                    current_page = current_page.map(|p| p.saturating_sub(1));
                                    needs_redraw = true;
                                }
                                _ => (),
                            }

                            let index = match keybinds.find(c) {
                                Some(i) => i + current_page.unwrap_or_default() * 10,
                                None => continue,
                            };

                            let dict = match dicts.get(index) {
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
                        view.change_mode(Mode::Normal);
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
