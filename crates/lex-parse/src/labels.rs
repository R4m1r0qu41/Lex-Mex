//! Shared grammar for Mexican federal article identifiers.
//!
//! Ported from the retired vault tooling's `article_labels.py` (see
//! `docs/decisions.md`, 2026-07-11). An identifier is one or more
//! components joined by horizontal whitespace or a single line break:
//! a base number with optional thousands separators or dotted segments,
//! an optional ordinal mark (`1o`, `2º`), an optional single-letter
//! suffix (`15-D`) that must never swallow a qualifier (`32-Bis`), and an
//! optional qualifier (`Bis` … `Octies`). Secondary regulations use
//! compound identifiers such as `2 Bis 102`, where every trailing numeric
//! component is part of the identifier.

/// Qualifier words in rank order; rank is index + 1.
const QUALIFIERS: [&str; 9] = [
    "Bis",
    "Ter",
    "Quáter",
    "Quater",
    "Quinquies",
    "Sexies",
    "Septies",
    "Octies",
    "Nonies",
];

/// Rank used for ordering; `Quáter` and `Quater` are the same qualifier.
fn qualifier_rank(index: usize) -> u8 {
    match index {
        0 => 1,     // Bis
        1 => 2,     // Ter
        2 | 3 => 3, // Quáter / Quater
        4 => 4,     // Quinquies
        5 => 5,     // Sexies
        6 => 6,     // Septies
        7 => 7,     // Octies
        _ => 8,     // Nonies
    }
}

const SUFFIX_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZÁÉÍÓÚÑ";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Component {
    /// Dot-separated numeric segments with thousands separators removed;
    /// almost always a single segment.
    number: Vec<u64>,
    /// `1o`, `2º`, `3°` ordinal mark (ignored for ordering).
    ordinal: bool,
    /// Single-letter suffix as written (`15-D`).
    letter: Option<char>,
    /// Qualifier rank (`Bis` = 1 … `Nonies` = 8).
    qualifier: Option<u8>,
    /// Qualifier as written, for the raw form.
    qualifier_text: Option<String>,
}

/// A parsed article identifier with its exact source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleLabel {
    raw: String,
    components: Vec<Component>,
}

/// Ordering key: components compare lexicographically as
/// `(number segments, letter rank, qualifier rank)`. A bare base sorts
/// before its suffixed forms (`15` < `15 Bis` < `15-A`), and a compound
/// identifier sorts after its prefix (`2 Bis` < `2 Bis 102`). When a
/// document mixes qualifier and letter suffixes on the same base, the
/// qualifier form sorts first; that choice is arbitrary but deterministic.
pub type ArticleSortKey = Vec<(Vec<u64>, u8, u8)>;

impl ArticleLabel {
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Filename/anchor slug, matching the retired tooling's
    /// `slugify_article_label`: `168 Bis 10` → `168-bis-10`,
    /// `1o.` → `1o`, `2º` → `2o`.
    #[must_use]
    pub fn slug(&self) -> String {
        slugify_label(&self.raw)
    }

    /// Slug with the ordinal mark canonicalized away, so `8` and `8o`
    /// (and `2º`, `2o`) are the same article.
    #[must_use]
    pub fn canonical_slug(&self) -> String {
        canonical_slug(&self.raw)
    }

    #[must_use]
    pub fn sort_key(&self) -> ArticleSortKey {
        self.components
            .iter()
            .map(|component| {
                (
                    component.number.clone(),
                    component.letter.map_or(0, letter_rank),
                    component.qualifier.unwrap_or(0),
                )
            })
            .collect()
    }
}

fn letter_rank(letter: char) -> u8 {
    SUFFIX_LETTERS
        .chars()
        .position(|candidate| candidate == letter)
        .map_or(u8::MAX, |index| u8::try_from(index + 1).unwrap_or(u8::MAX))
}

fn strip_accents(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'Á' | 'À' | 'Ä' => 'A',
            'É' | 'È' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Ü' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            _ => character,
        })
        .collect()
}

/// Slugify any article label text (whether or not it parses as an
/// `ArticleLabel`): accent-stripped, lowercased, ordinal marks `º`/`°`
/// normalized, thousands separators removed, every other non-alphanumeric
/// run collapsed to `-`.
#[must_use]
pub fn slugify_label(label: &str) -> String {
    let lowered = strip_accents(label).to_lowercase();
    let cleaned: String = lowered
        .chars()
        .filter(|character| *character != '°' && *character != 'º' && *character != ',')
        .collect();
    let mut slug = String::with_capacity(cleaned.len());
    let mut pending_dash = false;
    for character in cleaned.chars() {
        if character.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character);
        } else {
            pending_dash = true;
        }
    }
    slug
}

/// `slugify_label` with the trailing ordinal `o` removed from numeric
/// segments (`8o` → `8`, `2o-bis` → `2-bis`), matching the retired
/// tooling's `canonical_article_slug`.
#[must_use]
pub fn canonical_slug(label: &str) -> String {
    let slug = slugify_label(label);
    let mut canonical = String::with_capacity(slug.len());
    let characters: Vec<char> = slug.chars().collect();
    for (index, character) in characters.iter().enumerate() {
        if *character == 'o'
            && index > 0
            && characters[index - 1].is_ascii_digit()
            && characters.get(index + 1).is_none_or(|next| *next == '-')
        {
            continue;
        }
        canonical.push(*character);
    }
    canonical
}

struct Cursor<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Cursor<'a> {
    fn rest(&self) -> &'a str {
        &self.text[self.position..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self, character: char) {
        self.position += character.len_utf8();
    }

    /// Consume `[ \t]*` and return how many characters were consumed.
    fn eat_inline_space(&mut self) -> usize {
        let mut count = 0;
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.position += 1;
            count += 1;
        }
        count
    }

    /// Consume a component join: `[ \t]+` or `[ \t]*\n[ \t]*` (one line
    /// break at most — a blank line ends the identifier). Returns false
    /// (and restores the position) when there is no join.
    fn eat_join(&mut self) -> bool {
        let start = self.position;
        let spaces = self.eat_inline_space();
        if self.peek() == Some('\n') {
            self.position += 1;
            self.eat_inline_space();
            if self.peek() == Some('\n') {
                self.position = start;
                return false;
            }
            return true;
        }
        if spaces > 0 {
            return true;
        }
        self.position = start;
        false
    }
}

fn parse_number(cursor: &mut Cursor) -> Option<Vec<u64>> {
    let rest = cursor.rest();
    let mut segments = Vec::new();
    let mut digits = String::new();
    let mut consumed = 0;
    let bytes = rest.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_digit() {
            digits.push(byte as char);
            index += 1;
        } else if byte == b','
            && !digits.is_empty()
            && bytes.len() >= index + 4
            && bytes[index + 1..index + 4].iter().all(u8::is_ascii_digit)
            && bytes
                .get(index + 4)
                .is_none_or(|next| !next.is_ascii_digit())
        {
            // Thousands separator: exactly three digits follow.
            index += 1;
        } else if byte == b'.'
            && !digits.is_empty()
            && bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
        {
            segments.push(digits.parse().ok()?);
            digits.clear();
            index += 1;
        } else {
            break;
        }
        consumed = index;
    }
    if digits.is_empty() {
        return None;
    }
    segments.push(digits.parse().ok()?);
    cursor.position += consumed;
    Some(segments)
}

fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic() || "ÁÉÍÓÚÑáéíóúñ".contains(character)
}

fn parse_qualifier(cursor: &mut Cursor) -> Option<(u8, String)> {
    let start = cursor.position;
    // An ordinal-abbreviation dot may sit between the base and the
    // qualifier: `2o. Bis`, `2o. Ter`. Consume it only when it is
    // followed by inline whitespace (never `.-`, which introduces a
    // heading body, nor `.Bis`). The `start` rollback below undoes this
    // if no qualifier ultimately matches.
    if cursor.peek() == Some('.') && matches!(cursor.rest().as_bytes().get(1), Some(b' ' | b'\t')) {
        cursor.position += 1;
    }
    // Qualifier join: inline space, a single line break, or a hyphen.
    let mut joined = cursor.eat_join();
    if !joined {
        let spaces_start = cursor.position;
        cursor.eat_inline_space();
        if cursor.peek() == Some('-') {
            cursor.position += 1;
            cursor.eat_inline_space();
            joined = true;
        } else {
            cursor.position = spaces_start;
        }
    } else if cursor.peek() == Some('-') {
        cursor.position += 1;
        cursor.eat_inline_space();
    }
    if !joined {
        cursor.position = start;
        return None;
    }
    let rest = cursor.rest();
    for (index, qualifier) in QUALIFIERS.iter().enumerate() {
        if rest.len() >= qualifier.len()
            && rest
                .chars()
                .zip(qualifier.chars())
                .take(qualifier.chars().count())
                .all(|(actual, expected)| actual.to_lowercase().eq(expected.to_lowercase()))
        {
            let matched: String = rest.chars().take(qualifier.chars().count()).collect();
            let after = rest.chars().nth(qualifier.chars().count());
            if after.is_none_or(|next| !is_letter(next)) {
                cursor.position += matched.len();
                return Some((qualifier_rank(index), matched));
            }
        }
    }
    cursor.position = start;
    None
}

fn parse_component(cursor: &mut Cursor) -> Option<Component> {
    let number = parse_number(cursor)?;
    let mut ordinal = false;
    if let Some(mark) = cursor.peek()
        && (mark == 'o' || mark == 'º' || mark == '°')
    {
        // `1o`, `2º`: the mark binds only when not starting a word.
        let after = cursor.rest().chars().nth(1);
        if mark != 'o' || after.is_none_or(|next| !is_letter(next)) {
            cursor.bump(mark);
            ordinal = true;
        }
    }
    let mut letter = None;
    let letter_start = cursor.position;
    cursor.eat_inline_space();
    if cursor.peek() == Some('-') {
        cursor.position += 1;
        cursor.eat_inline_space();
        match cursor.peek() {
            Some(candidate)
                if SUFFIX_LETTERS.contains(candidate)
                    && cursor
                        .rest()
                        .chars()
                        .nth(1)
                        .is_none_or(|next| !is_letter(next)) =>
            {
                cursor.bump(candidate);
                letter = Some(candidate);
            }
            _ => cursor.position = letter_start,
        }
    } else {
        cursor.position = letter_start;
    }
    let (qualifier, qualifier_text) = match parse_qualifier(cursor) {
        Some((rank, text)) => (Some(rank), Some(text)),
        None => (None, None),
    };
    Some(Component {
        number,
        ordinal,
        letter,
        qualifier,
        qualifier_text,
    })
}

/// Parse the longest article label starting at the beginning of `text`.
#[must_use]
pub fn match_label_at(text: &str) -> Option<ArticleLabel> {
    let mut cursor = Cursor { text, position: 0 };
    let mut components = vec![parse_component(&mut cursor)?];
    let mut end = cursor.position;
    loop {
        if !cursor.eat_join() {
            // A hyphen also joins a trailing numeric component to a
            // qualifier: `270 Bis-1`, `270 Bis-2` (the space form
            // `270 Bis 1` already joins via `eat_join`). Require a digit
            // so a letter suffix or hyphenated qualifier, both consumed
            // inside `parse_component`, are never re-read here.
            if cursor.peek() == Some('-')
                && cursor
                    .rest()
                    .as_bytes()
                    .get(1)
                    .is_some_and(u8::is_ascii_digit)
            {
                cursor.position += 1;
            } else {
                break;
            }
        }
        let Some(component) = parse_component(&mut cursor) else {
            break;
        };
        components.push(component);
        end = cursor.position;
    }
    Some(ArticleLabel {
        raw: text[..end].to_string(),
        components,
    })
}

/// All article labels in `text`, scanned left to right, longest match at
/// each starting digit, non-overlapping.
#[must_use]
pub fn find_labels(text: &str) -> Vec<(std::ops::Range<usize>, ArticleLabel)> {
    let mut labels = Vec::new();
    let mut position = 0;
    while position < text.len() {
        let Some(offset) = text[position..].find(|character: char| character.is_ascii_digit())
        else {
            break;
        };
        let start = position + offset;
        match match_label_at(&text[start..]) {
            Some(label) => {
                let end = start + label.raw.len();
                labels.push((start..end, label));
                position = end;
            }
            None => position = start + 1,
        }
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::{canonical_slug, find_labels, match_label_at};

    fn full(text: &str) -> super::ArticleLabel {
        let label = match_label_at(text).expect("label parses");
        assert_eq!(label.raw(), text, "must consume the whole input");
        label
    }

    #[test]
    fn compound_secondary_identifier_is_one_label() {
        assert_eq!(full("168 Bis 10").slug(), "168-bis-10");
        assert_eq!(full("2 Bis 102").slug(), "2-bis-102");
        assert_eq!(full("32-Bis").slug(), "32-bis");
    }

    #[test]
    fn hyphen_joins_numeric_tail_to_qualifier() {
        // `270 Bis-1` / `270 Bis-2` (LCM) canonicalize like the space form.
        assert_eq!(full("270 Bis-1").slug(), "270-bis-1");
        assert_eq!(full("270 Bis-1").sort_key(), full("270 Bis 1").sort_key());
        assert_ne!(full("270 Bis-1").sort_key(), full("270 Bis-2").sort_key());
        assert!(full("270 Bis").sort_key() < full("270 Bis-1").sort_key());
    }

    #[test]
    fn letter_suffix_does_not_swallow_qualifiers() {
        let letter = full("32-A");
        assert_eq!(letter.slug(), "32-a");
        assert_eq!(full("32-B Bis").slug(), "32-b-bis");
        assert_eq!(full("15-D").slug(), "15-d");
    }

    #[test]
    fn article_list_remains_separate_labels() {
        let found = find_labels("1, 2 Bis y 3 Ter");
        let raws: Vec<&str> = found.iter().map(|(_, label)| label.raw()).collect();
        assert_eq!(raws, ["1", "2 Bis", "3 Ter"]);
    }

    #[test]
    fn ordinal_spellings_are_semantically_equivalent() {
        assert_eq!(canonical_slug("8"), canonical_slug("8o"));
        assert_eq!(canonical_slug("2º"), canonical_slug("2o"));
        assert_eq!(full("1o").canonical_slug(), "1");
        assert_eq!(full("1o").slug(), "1o");
    }

    #[test]
    fn ordinal_abbreviation_dot_precedes_qualifier() {
        // `Artículo 2o. Bis` (LFDO): the abbreviation dot must not strand
        // the qualifier, which would collapse `2o Bis` onto base `2`.
        assert_eq!(full("2o. Bis").slug(), "2o-bis");
        assert_eq!(full("2o. Ter").canonical_slug(), "2-ter");
        assert_ne!(full("2o. Bis").canonical_slug(), canonical_slug("2o"));
        // A bare heading body must not be read as a qualifier.
        let label = match_label_at("2o.- Cuando tres o más").expect("label parses");
        assert_eq!(label.raw(), "2o");
    }

    #[test]
    fn compound_identifier_does_not_cross_paragraph_boundary() {
        let label = match_label_at("1390 Bis\n\n49").expect("label parses");
        assert_eq!(label.raw(), "1390 Bis");
        let wrapped = match_label_at("1390 Bis\n49").expect("label parses");
        assert_eq!(wrapped.raw(), "1390 Bis\n49");
    }

    #[test]
    fn thousands_separators_are_part_of_the_number() {
        assert_eq!(full("1,390").slug(), "1390");
        assert_eq!(full("1,390").sort_key(), full("1390").sort_key());
    }

    #[test]
    fn sort_keys_order_suffixed_articles() {
        let base = full("15").sort_key();
        let bis = full("15 Bis").sort_key();
        let letter_a = full("15-A").sort_key();
        let letter_d = full("15-D").sort_key();
        let next = full("16").sort_key();
        let ordinal = full("15o").sort_key();
        assert!(base < bis && bis < letter_a && letter_a < letter_d && letter_d < next);
        assert_eq!(base, ordinal);
        let compound_prefix = full("2 Bis").sort_key();
        let compound = full("2 Bis 102").sort_key();
        assert!(compound_prefix < compound);
    }
}
