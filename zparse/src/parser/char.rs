use std::str::Chars;

use super::TryParse;

impl TryParse<Chars<'_>> for char {
    fn try_parse(mut input: Chars<'_>) -> Option<(Chars<'_>, Self)> {
        input.next().map(|char| (input, char))
    }
}
