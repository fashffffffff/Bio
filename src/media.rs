#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CheckStatus {
    Pending,
    Ok,
    Warn,
    Fail,
}

#[derive(Clone, Debug)]
pub struct MusicQueue {
    pub tracks: Vec<String>,
    pub index: usize,
}

impl Default for MusicQueue {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            index: 0,
        }
    }
}

impl MusicQueue {
    pub fn current(&self) -> Option<&str> {
        self.tracks.get(self.index).map(|s| s.as_str())
    }

    pub fn next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.tracks.len();
    }

    pub fn prev(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        self.index = (self.index + self.tracks.len() - 1) % self.tracks.len();
    }
}
