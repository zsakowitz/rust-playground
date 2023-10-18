pub mod parser;

use parser::{character::Character, Parse};
use zparse_derive::Parse;

fn main() -> Result<(), ()> {
    let x = Plus::parse("+".chars());

    println!("{x:?}");

    Ok(())
}

#[derive(Debug, Parse)]
pub enum MathToken {
    Plus(Character<'+'>),
    Sub(Character<'-'>),
    Times(Character<'*'>),
    Divide(Character<'/'>),
}

// #[derive(Debug, Parse)]
// pub struct Plus {
//     pub plus: Character<'+'>,
// }

// #[derive(Clone, Copy, Debug, Default, Hash, Parse, PartialEq, Eq)]
// pub struct Sub(Character<'-'>);

// #[derive(Clone, Copy, Debug, Default, Hash, Parse, PartialEq, Eq)]
// pub struct Mult(Character<'*'>);

// #[derive(Clone, Copy, Debug, Default, Hash, Parse, PartialEq, Eq)]
// pub struct Div(Character<'/'>);

// pub enum DecimalDigit {
//     D0(Character<'0'>),
//     D1(Character<'1'>),
//     D2(Character<'2'>),
//     D3(Character<'3'>),
//     D4(Character<'4'>),
//     D5(Character<'5'>),
//     D6(Character<'6'>),
//     D7(Character<'7'>),
//     D8(Character<'8'>),
//     D9(Character<'9'>),
// }
