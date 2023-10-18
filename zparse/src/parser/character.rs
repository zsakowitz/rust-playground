use super::Parse;
use std::str::Chars;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct Character<const T: char>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CharacterParseError {
    EndOfInput { expected: char },
    WrongCharacter { expected: char, actual: char },
}

impl<const T: char> Parse<Chars<'_>> for Character<T> {
    type Error = CharacterParseError;

    fn parse(mut chars: Chars) -> Result<(Chars, Self), Self::Error> {
        match chars.next() {
            None => Err(CharacterParseError::EndOfInput { expected: T }),

            Some(character) => {
                if character == T {
                    Ok((chars, Character))
                } else {
                    Err(CharacterParseError::WrongCharacter {
                        expected: T,
                        actual: character,
                    })
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
