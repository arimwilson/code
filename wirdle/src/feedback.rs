use crate::word::Word;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterStatus {
    Absent,
    Present,
    Correct,
}

impl LetterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Present => "present",
            Self::Correct => "correct",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim_matches('"').trim() {
            "absent" | "gray" | "grey" => Some(Self::Absent),
            "present" | "yellow" => Some(Self::Present),
            "correct" | "green" => Some(Self::Correct),
            _ => None,
        }
    }
}

pub type Pattern = u16;

pub fn pattern_from_statuses(statuses: [LetterStatus; 5]) -> Pattern {
    statuses
        .iter()
        .enumerate()
        .map(|(idx, status)| {
            let digit = match status {
                LetterStatus::Absent => 0,
                LetterStatus::Present => 1,
                LetterStatus::Correct => 2,
            };
            digit * 3u16.pow(idx as u32)
        })
        .sum()
}

pub fn statuses_from_pattern(pattern: Pattern) -> [LetterStatus; 5] {
    let mut output = [LetterStatus::Absent; 5];
    let mut remaining = pattern;
    for slot in &mut output {
        *slot = match remaining % 3 {
            0 => LetterStatus::Absent,
            1 => LetterStatus::Present,
            _ => LetterStatus::Correct,
        };
        remaining /= 3;
    }
    output
}

pub fn evaluate_feedback(guess: Word, answer: Word) -> Pattern {
    let mut statuses = [LetterStatus::Absent; 5];
    let mut answer_counts = [0u8; 26];

    for idx in 0..5 {
        if guess.0[idx] == answer.0[idx] {
            statuses[idx] = LetterStatus::Correct;
        } else {
            let answer_idx = usize::from(answer.0[idx] - b'a');
            answer_counts[answer_idx] += 1;
        }
    }

    for idx in 0..5 {
        if statuses[idx] == LetterStatus::Correct {
            continue;
        }
        let guess_idx = usize::from(guess.0[idx] - b'a');
        if answer_counts[guess_idx] > 0 {
            statuses[idx] = LetterStatus::Present;
            answer_counts[guess_idx] -= 1;
        }
    }

    pattern_from_statuses(statuses)
}
