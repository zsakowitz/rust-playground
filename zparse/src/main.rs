pub mod parser;

use parser::{character::Character, TryParse};
use vec1::Vec1;
use zparse_derive::TryParse;

type A = Character<'a'>;

fn main() -> Result<(), ()> {
    let x = <Vec1<Abc>>::try_parse("abcdef");

    println!("{x:?}");

    Ok(())
}

#[derive(Debug, TryParse)]
struct Abc {
    a: Character<'a'>,
    b: Character<'b'>,
    c: Character<'c'>,
}
