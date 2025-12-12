use std::path::{Path, PathBuf};

pub struct PathHistory {
    pub index: usize,
    pub buffer: Vec<PathBuf>,
}

impl PathHistory {
    pub fn new() -> Self {
        PathHistory {
            index: 0,
            buffer: Vec::new(),
        }
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        if self.index + 1 < self.buffer.len() {
            self.buffer.truncate(self.index + 1);
        }

        let new_path = path.as_ref().to_path_buf();
        if self.buffer.last() != Some(&new_path) {
            self.buffer.push(new_path);
        }

        self.index = self.buffer.len() - 1;
    }

    pub fn go_up(&mut self) -> Option<&PathBuf> {
        if self.index > 0 {
            self.index -= 1;
            return self.buffer.get(self.index);
        }

        None
    }

    pub fn go_down(&mut self) -> Option<&PathBuf> {
        if self.index < self.buffer.len() - 1 {
            self.index += 1;
            return self.buffer.get(self.index);
        }

        None
    }
}
