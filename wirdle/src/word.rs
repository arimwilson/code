use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Word(pub [u8; 5]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordError {
    WrongLength,
    NonAsciiLowercase,
}

impl Word {
    pub fn parse(input: &str) -> Result<Self, WordError> {
        let normalized = input.trim().to_ascii_lowercase();
        let bytes = normalized.as_bytes();
        if bytes.len() != 5 {
            return Err(WordError::WrongLength);
        }
        if !bytes.iter().all(|b| b.is_ascii_lowercase()) {
            return Err(WordError::NonAsciiLowercase);
        }
        let mut word = [0u8; 5];
        word.copy_from_slice(bytes);
        Ok(Self(word))
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("Word is validated lowercase ASCII")
    }

    pub fn letters(&self) -> [usize; 5] {
        self.0.map(|b| usize::from(b - b'a'))
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Word {
    type Err = WordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
