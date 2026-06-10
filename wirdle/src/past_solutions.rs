use crate::solver::PastSolutionPolicy;
use crate::word::Word;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct PastSolutionEntry {
    pub date: String,
    pub puzzle_number: u32,
    pub solution: Word,
    pub source: String,
    pub is_repeat: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PastSolutionIndex {
    entries: Vec<PastSolutionEntry>,
    latest_by_word: BTreeMap<Word, i64>,
    as_of_day: Option<i64>,
}

impl PastSolutionIndex {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err),
        };
        let mut entries = Vec::new();
        for object in json_objects(&text) {
            let date = json_string(object, "date").unwrap_or_default();
            let puzzle_number = json_number(object, "puzzle_number").unwrap_or_default() as u32;
            let source = json_string(object, "source").unwrap_or_default();
            let is_repeat = json_bool(object, "is_repeat").unwrap_or(false);
            let Some(solution_text) = json_string(object, "solution") else {
                continue;
            };
            let solution = Word::parse(&solution_text).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid past solution '{solution_text}': {err:?}"),
                )
            })?;
            entries.push(PastSolutionEntry {
                date,
                puzzle_number,
                solution,
                source,
                is_repeat,
            });
        }
        Ok(Self::from_entries(entries))
    }

    pub fn from_entries(entries: Vec<PastSolutionEntry>) -> Self {
        let as_of_day = entries
            .iter()
            .filter_map(|entry| civil_day(&entry.date))
            .max();
        let mut latest_by_word: BTreeMap<Word, i64> = BTreeMap::new();
        for entry in &entries {
            if let Some(day) = civil_day(&entry.date) {
                latest_by_word
                    .entry(entry.solution)
                    .and_modify(|existing| *existing = (*existing).max(day))
                    .or_insert(day);
            }
        }
        Self {
            entries,
            latest_by_word,
            as_of_day,
        }
    }

    pub fn entries(&self) -> &[PastSolutionEntry] {
        &self.entries
    }

    pub fn date_range(&self) -> Option<(&str, &str)> {
        let mut dates = self.entries.iter().map(|entry| entry.date.as_str());
        let first = dates.next()?;
        let (earliest, latest) = dates.fold((first, first), |(earliest, latest), date| {
            (earliest.min(date), latest.max(date))
        });
        Some((earliest, latest))
    }

    pub fn as_of_before(&self, date: &str) -> Self {
        let Some(cutoff) = civil_day(date) else {
            return Self::default();
        };
        Self::from_entries(
            self.entries
                .iter()
                .filter(|entry| civil_day(&entry.date).is_some_and(|day| day < cutoff))
                .cloned()
                .collect(),
        )
    }

    pub fn was_ever_solution(&self, word: Word) -> bool {
        self.latest_by_word.contains_key(&word)
    }

    pub fn was_recent_solution(&self, word: Word, recent_days: usize) -> bool {
        let Some(latest) = self.latest_by_word.get(&word) else {
            return false;
        };
        let Some(as_of_day) = self.as_of_day else {
            return false;
        };
        as_of_day - latest <= recent_days as i64
    }

    pub fn multiplier(&self, word: Word, policy: &PastSolutionPolicy) -> f64 {
        if !policy.enabled {
            return 1.0;
        }
        if self.was_recent_solution(word, policy.recent_days) {
            policy.recent_repeat_multiplier
        } else if self.was_ever_solution(word) {
            policy.weight_multiplier
        } else {
            1.0
        }
    }
}

fn json_objects(text: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(&text[start..=idx]);
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn json_string(object: &str, key: &str) -> Option<String> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start;
    let first_quote = object[colon..].find('"')? + colon + 1;
    let second_quote = object[first_quote..].find('"')? + first_quote;
    Some(object[first_quote..second_quote].to_string())
}

fn json_number(object: &str, key: &str) -> Option<u64> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start + 1;
    let digits: String = object[colon..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn json_bool(object: &str, key: &str) -> Option<bool> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start + 1;
    let value: String = object[colon..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect();
    match value.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn civil_day(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    days_from_civil(year, month, day)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_i = month as i32;
    let doy = (153 * (month_i + if month_i > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146097 + doe - 719468) as i64)
}
