//! Deterministic text extraction for HTML formal-publication sources such as
//! Diario Oficial de la Federación notes. The extractor is intentionally
//! narrow: it handles block-level structure, table cells, and the character
//! entities that occur in DOF documents, and it never fetches subresources.

use regex::Regex;

/// Extract plain text from an HTML document.
///
/// Rules, in order:
/// 1. Bytes are decoded as Latin-1 (the DOF declares `iso-8859-1`).
/// 2. `<script>` and `<style>` elements are removed with their content.
/// 3. Closing block tags (`div`, `p`, `tr`, `table`, headings, `li`) become
///    newlines, `<br>` becomes a space (DOF uses one block element per
///    paragraph or table row), and closing cell tags become an internal cell
///    separator that is later rendered as ` | `.
/// 4. All remaining tags are stripped.
/// 5. Character entities are decoded (numeric plus the named set used by DOF
///    documents); unknown entities are preserved verbatim.
/// 6. Whitespace inside each line is collapsed, empty cells and lines are
///    dropped, and remaining lines are joined with single newlines.
///
/// # Panics
///
/// Never in practice: the only panics are `expect` calls on static, valid
/// regular expressions.
#[must_use]
pub fn extract_html_text(bytes: &[u8]) -> String {
    let raw: String = bytes.iter().map(|byte| char::from(*byte)).collect();
    let script_re = Regex::new(r"(?is)<(script|style)\b.*?</(script|style)>").expect("static");
    let linebreak_re = Regex::new(r"(?i)<br[^>]*>").expect("static");
    let break_re = Regex::new(r"(?i)</(div|p|tr|table|h[1-6]|li)>").expect("static");
    let cell_re = Regex::new(r"(?i)</t[dh]>").expect("static");
    let tag_re = Regex::new(r"(?s)<[^>]*>").expect("static");

    let text = script_re.replace_all(&raw, "");
    let text = linebreak_re.replace_all(&text, " ");
    let text = cell_re.replace_all(&text, "\u{1f}");
    let text = break_re.replace_all(&text, "\n");
    let text = tag_re.replace_all(&text, "");
    let text = decode_entities(&text);

    let mut lines = Vec::new();
    for line in text.lines() {
        let cells: Vec<String> = line
            .split('\u{1f}')
            .map(|cell| cell.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|cell| !cell.is_empty())
            .collect();
        if !cells.is_empty() {
            lines.push(cells.join(" | "));
        }
    }
    lines.join("\n")
}

fn decode_entities(value: &str) -> String {
    let entity_re = Regex::new(r"&(#x?[0-9a-fA-F]+|[a-zA-Z]+);").expect("static");
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    for capture in entity_re.captures_iter(value) {
        let whole = capture.get(0).expect("entity match");
        output.push_str(&value[cursor..whole.start()]);
        match decode_entity(&capture[1]) {
            Some(decoded) => output.push(decoded),
            None => output.push_str(whole.as_str()),
        }
        cursor = whole.end();
    }
    output.push_str(&value[cursor..]);
    output
}

fn decode_entity(name: &str) -> Option<char> {
    if let Some(numeric) = name.strip_prefix('#') {
        let code = if let Some(hex) = numeric.strip_prefix(['x', 'X']) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            numeric.parse().ok()?
        };
        return char::from_u32(code);
    }
    let decoded = match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "aacute" => 'á',
        "eacute" => 'é',
        "iacute" => 'í',
        "oacute" => 'ó',
        "uacute" => 'ú',
        "ntilde" => 'ñ',
        "uuml" => 'ü',
        "Aacute" => 'Á',
        "Eacute" => 'É',
        "Iacute" => 'Í',
        "Oacute" => 'Ó',
        "Uacute" => 'Ú',
        "Ntilde" => 'Ñ',
        "Uuml" => 'Ü',
        "iexcl" => '¡',
        "iquest" => '¿',
        "laquo" => '«',
        "raquo" => '»',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "hellip" => '\u{2026}',
        "ordm" => 'º',
        "ordf" => 'ª',
        "deg" => '°',
        "sect" => '§',
        "middot" => '·',
        "times" => '×',
        "eq" => '=',
        _ => return None,
    };
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::extract_html_text;

    #[test]
    fn extracts_block_text_cells_and_entities() {
        let html = b"<html><head><script>var x = '<div>';</script></head><body>\
<div>Primer p&aacute;rrafo.</div>\
<table><tr><td>Tipo</td><td>Definici&oacute;n</td></tr>\
<tr><td>I. Fraude</td><td>P&eacute;rdidas &#233;ticas</td></tr></table>\
<p>Cierre &ntilde;&nbsp; final &desconocida;</p></body></html>";
        assert_eq!(
            extract_html_text(html),
            "Primer párrafo.\nTipo | Definición\nI. Fraude | Pérdidas éticas\nCierre ñ final &desconocida;"
        );
    }
}
