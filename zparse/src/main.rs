pub mod parser;

use parser::{character::Character, TryParse};
use vec1::Vec1;
use zparse_derive::TryParse;

fn main() -> Result<(), ()> {
    let x = Atom::try_parse("2832+45");

    println!("{x:?}");

    Ok(())
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, TryParse)]
pub enum Digit {
    D0(Character<'0'>),
    D1(Character<'1'>),
    D2(Character<'2'>),
    D3(Character<'3'>),
    D4(Character<'4'>),
    D5(Character<'5'>),
    D6(Character<'6'>),
    D7(Character<'7'>),
    D8(Character<'8'>),
    D9(Character<'9'>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Numeral(pub Vec1<Digit>);

impl<T: Clone> TryParse<T> for Numeral
where
    Digit: TryParse<T>,
{
    fn try_parse(input: T) -> Option<(T, Self)> {
        TryParse::try_parse(input).map(|(input, value)| (input, Numeral(value)))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Atom {
    Number(Numeral),
    Parenthesized(Box<Atom>),
}

impl<T: Clone> TryParse<T> for Atom
where
    Numeral: TryParse<T>,
    Box<Atom>: TryParse<T>,
{
    fn try_parse(input: T) -> Option<(T, Self)> {
        if let Some((input, value)) = TryParse::try_parse(input.clone()) {
            return Some((input, Atom::Number(value)));
        }

        if let Some((input, value)) = TryParse::try_parse(input) {
            return Some((input, Atom::Parenthesized(value)));
        }

        None
    }
}

// #[derive(Clone, Debug, Hash, PartialEq, Eq, TryParse)]
// pub struct MultiplicationOrDivision(Atom, Vec<(Either<Character<'*'>, Character<'/'>>, Atom)>);

// #[derive(Clone, Debug, Hash, PartialEq, Eq, TryParse)]
// pub struct AdditionOrSubtraction(
//     MultiplicationOrDivision,
//     Vec<(
//         Either<Character<'+'>, Character<'-'>>,
//         MultiplicationOrDivision,
//     )>,
// );
