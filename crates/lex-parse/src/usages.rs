//! Defined-term usage matching over supplied provisions and terms.

use anyhow::Result;
use lex_core::{DefinedTerm, Provision, SCHEMA_VERSION, TermUsage};

use crate::grammar::is_connector;

/// Extract every exact defined-term occurrence across the provisions.
///
/// `term_sets` is ordered by resolution priority: the instrument's own
/// glossary first, then the glossaries it is expressly additive to. At any
/// text position the longest matching variant wins; on equal length the
/// earlier term set wins. A term never matches inside its own definition
/// entry, and no term matches inside an entry's printed name, which is the
/// definition site, not a usage.
pub fn extract_term_usages(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
) -> Result<Vec<TermUsage>> {
    Ok(term_usages(provisions, term_sets, false))
}

/// [`extract_term_usages`], where each term also matches in lower case
/// (`tabacos labrados` for `Tabacos labrados`). An adapter opts in with
/// `usage_form: "with_lowercase"` after review, for an instrument whose
/// running text writes its defined terms in lower case (LIEPS); the
/// exact-case form still wins a tie.
pub fn extract_term_usages_with_lowercase(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
) -> Result<Vec<TermUsage>> {
    Ok(term_usages(provisions, term_sets, true))
}

fn term_usages(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
    lowercase: bool,
) -> Vec<TermUsage> {
    let mut candidates: Vec<(String, &DefinedTerm)> = Vec::new();
    for set in term_sets {
        for term in *set {
            let variants = term_variants(&term.term);
            let lower: Vec<String> = if lowercase {
                variants
                    .iter()
                    .map(|variant| variant.to_lowercase())
                    .filter(|variant| !variants.contains(variant))
                    .collect()
            } else {
                Vec::new()
            };
            for variant in variants.into_iter().chain(lower) {
                candidates.push((variant, term));
            }
        }
    }
    // Longest variants first; ties keep `term_sets` priority order because
    // the sort is stable.
    candidates.sort_by_key(|(variant, _)| std::cmp::Reverse(variant.chars().count()));

    let mut usages = Vec::new();
    for provision in provisions {
        usages.extend(provision_usages(provision, &candidates));
    }
    usages
}

fn provision_usages(
    provision: &Provision,
    candidates: &[(String, &DefinedTerm)],
) -> Vec<TermUsage> {
    let chars: Vec<char> = provision.text.chars().collect();
    let names = defined_name_spans(&chars, provision, candidates);
    let mut usages = Vec::new();
    let mut position = 0;
    while position < chars.len() {
        if !starts_word(&chars, position) {
            position += 1;
            continue;
        }
        if let Some(&(_, end)) = names
            .iter()
            .find(|(start, end)| (*start..*end).contains(&position))
        {
            position = end;
            continue;
        }
        let mut matched = None;
        for (variant, term) in candidates {
            if matches_at(&chars, position, variant)
                && !inside_own_definition(term, provision, position)
                && capitalization_is_informative(&chars, position, variant)
            {
                matched = Some((variant.chars().count(), *term));
                break;
            }
        }
        let Some((length, term)) = matched else {
            position += 1;
            continue;
        };
        let end_char = position + length;
        let span: String = chars[position..end_char].iter().collect();
        usages.push(TermUsage {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:term-usage:{position}-{end_char}", provision.id),
            provision_id: provision.id.clone(),
            term_id: term.id.clone(),
            span,
            start_char: position,
            end_char,
        });
        position = end_char;
    }
    usages
}

/// Extract usages written exactly as a glossary prints its vocables: the
/// term, case-sensitive and without singular/plural variants, between a
/// matched pair of straight (`"Ley"`) or curly (`“Ley”`) quotes. The usage
/// span is the term inside the quotes. An adapter opts in with
/// `usage_form: "quoted"` (`reg-lan`, whose Article 1 defines its vocables
/// in quotes), so a bare word, or the first word of another title such as
/// `Ley Federal de Derechos`, is not a usage. `term_sets` priority and the
/// own-definition exclusion are as in [`extract_term_usages`].
pub fn extract_quoted_term_usages(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
) -> Result<Vec<TermUsage>> {
    let mut candidates: Vec<&DefinedTerm> = term_sets.iter().flat_map(|set| set.iter()).collect();
    // Longest terms first; ties keep `term_sets` priority (stable sort).
    candidates.sort_by_key(|term| std::cmp::Reverse(term.term.chars().count()));

    let mut usages = Vec::new();
    for provision in provisions {
        let chars: Vec<char> = provision.text.chars().collect();
        let mut position = 0;
        while position < chars.len() {
            let close = match chars[position] {
                '"' => '"',
                '“' => '”',
                _ => {
                    position += 1;
                    continue;
                }
            };
            let start = position + 1;
            let matched = candidates.iter().find(|term| {
                let end = start + term.term.chars().count();
                matches_at(&chars, start, &term.term)
                    && chars.get(end) == Some(&close)
                    && !inside_own_definition(term, provision, start)
            });
            let Some(term) = matched else {
                position += 1;
                continue;
            };
            let end_char = start + term.term.chars().count();
            usages.push(TermUsage {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{}:term-usage:{start}-{end_char}", provision.id),
                provision_id: provision.id.clone(),
                term_id: term.id.clone(),
                span: term.term.clone(),
                start_char: start,
                end_char,
            });
            // Resume after the closing quote, so it never opens a new match.
            position = end_char + 1;
        }
    }
    Ok(usages)
}

fn starts_word(chars: &[char], position: usize) -> bool {
    chars[position].is_alphabetic() && (position == 0 || !chars[position - 1].is_alphanumeric())
}

fn matches_at(chars: &[char], position: usize, variant: &str) -> bool {
    let mut end = position;
    for expected in variant.chars() {
        if chars.get(end) != Some(&expected) {
            return false;
        }
        end += 1;
    }
    // Word boundary after the match.
    chars.get(end).is_none_or(|next| !next.is_alphanumeric())
}

/// The printed names this provision's glossary entries define, as
/// `[start, end)` character spans: each term's first occurrence in its own
/// entry. A name is its definition site, so no term matches inside it
/// (`Uso` in `"Uso Agrícola"`, `Cliente` in `Identificador de Cliente`).
/// The definition text after the name is ordinary text.
fn defined_name_spans(
    chars: &[char],
    provision: &Provision,
    candidates: &[(String, &DefinedTerm)],
) -> Vec<(usize, usize)> {
    let mut names: Vec<(usize, usize)> = candidates
        .iter()
        .filter(|(_, term)| term.defining_provision_id == provision.id)
        .filter_map(|(_, term)| {
            let name: Vec<char> = term.term.chars().collect();
            (term.start_char..term.end_char.min(chars.len()))
                .find(|&start| chars[start..].starts_with(&name))
                .map(|start| (start, start + name.len()))
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

fn inside_own_definition(term: &DefinedTerm, provision: &Provision, position: usize) -> bool {
    provision.id == term.defining_provision_id
        && position >= term.start_char
        && position < term.end_char
}

/// Defined-term matching is case-sensitive because capitalization is what
/// distinguishes the defined `Control` from the ordinary word `control`.
/// At a sentence, list-item, or table-cell start the capital is positional
/// and carries no signal, so a term whose only capital is its initial
/// letter does not match there (`I. Controles de acceso…` is not the
/// defined `Control`). Terms with capitals beyond the first character —
/// acronyms such as `CNBV`, multi-word terms such as `Autoridad
/// Financiera` — remain informative anywhere.
fn capitalization_is_informative(chars: &[char], position: usize, variant: &str) -> bool {
    if !only_initial_capital(variant) {
        return true;
    }
    let mut index = position;
    while index > 0 && chars[index - 1].is_whitespace() {
        index -= 1;
    }
    if index == 0 {
        return false;
    }
    !matches!(chars[index - 1], '.' | ':' | ';' | ')' | '|')
}

fn only_initial_capital(variant: &str) -> bool {
    let mut characters = variant.chars();
    characters.next().is_some_and(char::is_uppercase)
        && characters.all(|character| !character.is_uppercase())
}

/// Generate the deterministic singular/plural variants of a term. Glossary
/// provisions state that terms apply "en singular o plural", so a term
/// defined as `Operaciones` must match `Operación` and vice versa. Rules,
/// applied word by word (skipping short connector words):
/// `-ón` ↔ `-ones`; vowel ending ↔ `+s`; consonant ending ↔ `+es`.
fn term_variants(term: &str) -> Vec<String> {
    let mut variants = vec![term.to_owned()];
    for transformed in [
        transform_words(term, pluralize),
        transform_words(term, singularize),
    ] {
        if let Some(value) = transformed
            && !variants.contains(&value)
        {
            variants.push(value);
        }
    }
    variants
}

fn transform_words(term: &str, transform: fn(&str) -> Option<String>) -> Option<String> {
    let words: Vec<&str> = term.split(' ').collect();
    let mut output = Vec::with_capacity(words.len());
    let mut changed = false;
    for word in words {
        if is_connector(word) {
            output.push(word.to_owned());
            continue;
        }
        match transform(word) {
            Some(transformed) => {
                changed = true;
                output.push(transformed);
            }
            None => output.push(word.to_owned()),
        }
    }
    changed.then(|| output.join(" "))
}

fn pluralize(word: &str) -> Option<String> {
    let lower_end = word.chars().last()?;
    if word.chars().count() < 3 || word.ends_with('s') || word.ends_with('S') {
        return None;
    }
    if let Some(stem) = word.strip_suffix("ón") {
        return Some(format!("{stem}ones"));
    }
    if matches!(
        lower_end,
        'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú'
    ) {
        return Some(format!("{word}s"));
    }
    if lower_end.is_alphabetic() {
        return Some(format!("{word}es"));
    }
    None
}

fn singularize(word: &str) -> Option<String> {
    if word.chars().count() < 4 {
        return None;
    }
    if let Some(stem) = word.strip_suffix("ones") {
        return Some(format!("{stem}ón"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        // Only strip `-es` after a consonant that takes it (`-dades`,
        // `-ores`…); vowel stems take plain `-s`.
        if stem
            .chars()
            .last()
            .is_some_and(|last| matches!(last, 'd' | 'l' | 'r' | 'n' | 'j'))
        {
            return Some(stem.to_owned());
        }
    }
    if let Some(stem) = word.strip_suffix('s')
        && stem.chars().last().is_some_and(|last| {
            matches!(
                last,
                'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú'
            )
        })
    {
        return Some(stem.to_owned());
    }
    None
}
