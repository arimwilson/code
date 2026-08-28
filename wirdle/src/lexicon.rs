use crate::word::Word;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct EditorialOverrides {
    pub add_likelier_solutions: BTreeSet<Word>,
    pub remove_likelier_solutions: BTreeSet<Word>,
    pub word_priors: BTreeMap<Word, f64>,
}

impl EditorialOverrides {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err),
        };
        Ok(Self {
            // `*_candidate_solutions` are the pre-v2.1 key names. Accept them so
            // an un-migrated overrides file keeps working instead of being
            // silently ignored and then overwritten by the nightly updater.
            add_likelier_solutions: parse_word_array(&text, "add_likelier_solutions")
                .into_iter()
                .chain(parse_word_array(&text, "add_candidate_solutions"))
                .collect(),
            remove_likelier_solutions: parse_word_array(&text, "remove_likelier_solutions")
                .into_iter()
                .chain(parse_word_array(&text, "remove_candidate_solutions"))
                .collect(),
            word_priors: parse_word_priors(&text),
        })
    }
}

/// Ranking weight for words on the likelier-solutions list.
const LIKELIER_WEIGHT: f64 = 1.0;

/// Ranking weight for accepted words that are not on the likelier-solutions
/// list. NYT picks these regularly now (8 of the ~45 answers after
/// 2026-07-15), so they must stay live candidates, just less likely ones.
const OTHER_ACCEPTED_WEIGHT: f64 = 0.2;

#[derive(Clone, Debug)]
pub struct Lexicon {
    /// The full NYT accepted-guess list. This is also the solution universe:
    /// any accepted word can be the answer.
    pub allowed_guesses: Vec<Word>,
    /// Words NYT has historically drawn answers from. Used only as a ranking
    /// prior, never as a filter.
    pub likelier_solutions: BTreeSet<Word>,
    pub overrides: EditorialOverrides,
}

impl Lexicon {
    pub fn load(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        let data_dir = data_dir.as_ref();
        let mut allowed_guesses = load_words(data_dir.join("allowed_guesses.txt"))?;
        let mut likelier_solutions: BTreeSet<Word> =
            load_words(data_dir.join("likelier_solutions.txt"))?
                .into_iter()
                .collect();
        let overrides = EditorialOverrides::load(data_dir.join("editorial_overrides.json"))?;

        for word in &overrides.add_likelier_solutions {
            likelier_solutions.insert(*word);
            if !allowed_guesses.contains(word) {
                allowed_guesses.push(*word);
            }
        }
        likelier_solutions.retain(|word| !overrides.remove_likelier_solutions.contains(word));
        sort_dedup(&mut allowed_guesses);

        Ok(Self {
            allowed_guesses,
            likelier_solutions,
            overrides,
        })
    }

    /// Ranking weight for `word`, by whether NYT has historically drawn
    /// answers from it.
    pub fn likelier_weight(&self, word: Word) -> f64 {
        if self.likelier_solutions.contains(&word) {
            LIKELIER_WEIGHT
        } else {
            OTHER_ACCEPTED_WEIGHT
        }
    }

    pub fn is_likelier(&self, word: Word) -> bool {
        self.likelier_solutions.contains(&word)
    }
}

pub fn load_words(path: impl AsRef<Path>) -> io::Result<Vec<Word>> {
    let text = fs::read_to_string(path)?;
    let mut words = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let word = Word::parse(line).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid word '{line}': {err:?}"),
            )
        })?;
        words.push(word);
    }
    sort_dedup(&mut words);
    Ok(words)
}

fn sort_dedup(words: &mut Vec<Word>) {
    words.sort_unstable();
    words.dedup();
}

fn parse_word_array(text: &str, key: &str) -> BTreeSet<Word> {
    let Some(start) = text.find(&format!("\"{key}\"")) else {
        return BTreeSet::new();
    };
    let Some(open_rel) = text[start..].find('[') else {
        return BTreeSet::new();
    };
    let open = start + open_rel;
    let Some(close_rel) = text[open..].find(']') else {
        return BTreeSet::new();
    };
    text[open + 1..open + close_rel]
        .split(',')
        .filter_map(|part| Word::parse(part.trim().trim_matches('"')).ok())
        .collect()
}

fn parse_word_priors(text: &str) -> BTreeMap<Word, f64> {
    let Some(start) = text.find("\"word_priors\"") else {
        return BTreeMap::new();
    };
    let Some(open_rel) = text[start..].find('{') else {
        return BTreeMap::new();
    };
    let open = start + open_rel;
    let Some(close_rel) = text[open + 1..].find('}') else {
        return BTreeMap::new();
    };
    let body = &text[open + 1..open + 1 + close_rel];
    let mut priors = BTreeMap::new();
    for entry in body.split(',') {
        let mut parts = entry.splitn(2, ':');
        let Some(word) = parts.next() else { continue };
        let Some(value) = parts.next() else { continue };
        if let (Ok(word), Ok(value)) = (
            Word::parse(word.trim().trim_matches('"')),
            value.trim().parse::<f64>(),
        ) {
            priors.insert(word, value);
        }
    }
    priors
}
