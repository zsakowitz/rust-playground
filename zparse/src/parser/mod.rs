pub mod character;
pub mod either;
pub mod option;
pub mod tuple;

pub trait Parse<T>: Sized {
    type Error;

    fn parse(input: T) -> Result<(T, Self), Self::Error>;

    fn parse_value(input: T) -> Result<Self, Self::Error> {
        Parse::parse(input).map(|(_, value)| value)
    }
}
