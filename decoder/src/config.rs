#[derive(Clone, Debug)]
pub struct Config {
    max_suffix_len: u64,
    max_string_len: u64,
    max_compressed_size: u64,
    max_uncompressed_size: u64,
    max_array_size: u64,
    max_hash_size: u64,
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
            max_string_len: 1_000_000,
            max_compressed_size: 100_000_000,
            max_uncompressed_size: 100_000_000,
            max_array_size: 1_000_000,
            max_hash_size: 1_000_000,
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

    pub fn max_string_len(&self) -> u64 {
        self.max_string_len
    }

    pub fn with_max_string_len(self, new_max: u64) -> Config {
        Config {
            max_string_len: new_max,
            ..self
        }
    }

    pub fn max_compressed_size(&self) -> u64 {
        self.max_compressed_size
    }

    pub fn with_max_compressed_size(self, new_max: u64) -> Config {
        Config {
            max_compressed_size: new_max,
            ..self
        }
    }

    pub fn max_uncompressed_size(&self) -> u64 {
        self.max_uncompressed_size
    }

    pub fn with_max_uncompressed_size(self, new_max: u64) -> Config {
        Config {
            max_uncompressed_size: new_max,
            ..self
        }
    }

    pub fn max_array_size(&self) -> u64 {
        self.max_array_size
    }

    pub fn with_max_array_size(self, new_max: u64) -> Config {
        Config {
            max_array_size: new_max,
            ..self
        }
    }

    pub fn max_hash_size(&self) -> u64 {
        self.max_hash_size
    }

    pub fn with_max_hash_size(self, new_max: u64) -> Config {
        Config {
            max_hash_size: new_max,
            ..self
        }
    }
}
