#[derive(Debug)]
pub struct Config {
    max_suffix_len: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config::default()
    }
}

impl Config {
    pub fn default() -> Config {
        Config {
            max_suffix_len: 1_000_000,
        }
    }

    pub fn max_suffix_len(&self) -> u64 {
        self.max_suffix_len
    }

    pub fn with_max_suffix_len(self, new_max: u64) -> Config {
        Config {
            max_suffix_len: new_max,
            ..self
        }
    }
}
