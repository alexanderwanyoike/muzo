use std::borrow::Cow;

pub(super) fn replace_embedded_breaks<'a>(label: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let label = label.into();
    if label.contains(embedded_break) {
        Cow::Owned(label.replace(embedded_break, " "))
    } else {
        label
    }
}

pub(super) fn single_line(label: &str) -> String {
    label
        .split(|character: char| character.is_ascii_whitespace() || embedded_break(character))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn embedded_break(character: char) -> bool {
    // These separators can reach the shaper inside a single CR/LF-delimited line.
    matches!(
        character,
        '\u{001c}'..='\u{001e}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}
