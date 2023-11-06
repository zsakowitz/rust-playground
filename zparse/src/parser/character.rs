use super::TryParse;
use std::str::Chars;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct Character<const T: char>;

impl<const T: char> TryParse<Chars<'_>> for Character<T> {
    fn try_parse(mut chars: Chars) -> Option<(Chars, Self)> {
        match chars.next() {
            None => None,
            Some(char) => {
                if char == T {
                    Some((chars, Self))
                } else {
                    None
                }
            }
        }
    }
}

impl<const T: char> TryParse<&str> for Character<T> {
    fn try_parse(input: &str) -> Option<(&str, Self)> {
        let mut chars = input.chars();

        match chars.next() {
            None => None,
            Some(char) => {
                if char == T {
                    Some((&input[char.len_utf8()..], Character))
                } else {
                    None
                }
            }
        }
    }
}

impl<const T: char> TryParse<String> for Character<T> {
    fn try_parse(input: String) -> Option<(String, Self)> {
        let mut chars = input.chars();

        match chars.next() {
            None => None,
            Some(char) => {
                if char == T {
                    Some((chars.collect(), Character))
                } else {
                    None
                }
            }
        }
    }
}

impl<const T: char> ToString for Character<T> {
    fn to_string(&self) -> String {
        T.to_string()
    }
}

impl<const T: char> From<Character<T>> for char {
    fn from(_: Character<T>) -> Self {
        T
    }
}

impl<const T: char> From<Character<T>> for String {
    fn from(_: Character<T>) -> Self {
        T.to_string()
    }
}
