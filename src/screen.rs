use crossterm::{
    cursor::{Hide, MoveDown, MoveToColumn, MoveUp, Show},
    execute,
    terminal::{Clear, ClearType, size},
};
use std::{
    fmt::Display,
    io::{Stderr, Write, stderr},
};
use strip_ansi_escapes::strip;

pub struct Screen {
    previous_num_rows: usize,
    line_lengths: Vec<usize>,
    stderr: Stderr,
}

impl Screen {
    pub fn new() -> Self {
        let stderr = stderr();
        Screen {
            previous_num_rows: 0,
            line_lengths: Vec::new(),
            stderr,
        }
    }

    pub fn write<T: Display>(&mut self, s: T) -> std::io::Result<()> {
        write!(self.stderr, "{}", s)?;

        let length = self.display_to_visible_length(&s);
        self.line_lengths.push(length);

        //eprintln!("{}", length);

        execute!(self.stderr, Clear(ClearType::UntilNewLine),)?;
        writeln!(self.stderr)?;

        Ok(())
    }

    pub fn empty_line(&mut self) -> std::io::Result<()> {
        execute!(self.stderr, Clear(ClearType::UntilNewLine),)?;
        writeln!(self.stderr)?;

        self.line_lengths.push(1);
        Ok(())
    }

    pub fn show_cursor(&mut self) -> std::io::Result<()> {
        execute!(self.stderr, Show)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> std::io::Result<()> {
        execute!(self.stderr, Hide)?;
        Ok(())
    }

    // Clears everything behind num_rows based on how many line_lengths there are.
    pub fn clear_rest(&mut self) -> std::io::Result<()> {
        let current_num_rows = self.calculate_num_rows()?;

        if self.previous_num_rows > current_num_rows {
            for _ in current_num_rows..self.previous_num_rows {
                execute!(self.stderr, Clear(ClearType::CurrentLine), MoveDown(1))?
            }
            execute!(
                self.stderr,
                MoveUp((self.previous_num_rows - current_num_rows) as u16)
            )?;
        }

        Ok(())
    }

    // Moves the cursor to the top based on calculated number of rows.
    pub fn move_up(&mut self) -> std::io::Result<()> {
        self.previous_num_rows = self.calculate_num_rows()?;
        self.line_lengths.clear();

        for _ in 0..self.previous_num_rows {
            execute!(self.stderr, MoveUp(1),)?;
        }
        execute!(self.stderr, MoveToColumn(0))?;
        self.stderr.flush()?;
        Ok(())
    }

    fn calculate_num_rows(&mut self) -> std::io::Result<usize> {
        if self.line_lengths.is_empty() {
            return Ok(0);
        }

        let (num_columns, _) = size()?;

        let mut num_rows: usize = 0;

        for &line_length in &self.line_lengths {
            let rows_per_line = line_length.div_ceil(num_columns as usize);
            num_rows += rows_per_line;
        }

        //eprintln!("{:?}", &self.line_lengths);
        //eprintln!("{}", num_columns);
        //eprintln!("{}", num_rows);

        Ok(num_rows)
    }

    fn display_to_visible_length(&mut self, d: impl Display) -> usize {
        let styled_string = d.to_string();
        let stripped_bytes = strip(&styled_string);
        let stripped_string = String::from_utf8(stripped_bytes).expect("Invalid UTF-8");

        stripped_string.chars().count()
    }
}
