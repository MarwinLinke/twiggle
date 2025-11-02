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
use visualize::{Mode, print_screen};

fn main() -> io::Result<()> {
    let _raw_mode = RawModeGuard::new();

    let keybinds = "1234567890";

    let mut screen = Screen::new();
    let mut mode = Mode::Normal;
    let mut ambiguous_char: Option<char> = None;

    loop {
        disable_raw_mode()?;
        screen.hide_cursor()?;
        screen.move_up()?;

        let current_dir = std::env::current_dir()?;
        let (char_map, files) = get_dirs()?;

        //eprintln!("{:?}", char_map);

        print_screen(
            &mut screen,
            &current_dir,
            &char_map,
            &files,
            &mode,
            ambiguous_char,
            keybinds,
        )?;
        screen.clear_rest()?;
        enable_raw_mode()?;
        screen.show_cursor()?;

        if let Event::Key(e) = event::read()? {
            match e.code {
                KeyCode::Char(c) => {
                    if c == '~' {
                        let home_dir = dirs::home_dir().expect("Could not find home directory");
                        env::set_current_dir(&home_dir)?;
                        mode = Mode::Normal;
                        continue;
                    }

                    match mode {
                        Mode::Normal => {
                            let dict_entries = match char_map.get(&c) {
                                Some(entry) => entry,
                                None => continue, // early exit if not found
                            };

                            if dict_entries.len() > 1 {
                                mode = Mode::Sub;
                                ambiguous_char = Some(c);
                                continue;
                            } else {
                                mode = Mode::Normal
                            }

                            env::set_current_dir(dict_entries[0].file_name().unwrap())?;
                        }
                        Mode::Sub => {
                            let index = match keybinds.find(c) {
                                Some(i) => i,
                                None => continue,
                            };

                            let dicts = char_map.get(&ambiguous_char.unwrap()).unwrap();
                            let dict = match dicts.get(index) {
                                Some(dict) => dict,
                                None => continue,
                            };

                            env::set_current_dir(dict.file_name().unwrap())?;

                            mode = Mode::Normal;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if matches!(mode, Mode::Sub) {
                        mode = Mode::Normal;
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
