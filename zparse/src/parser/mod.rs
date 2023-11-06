pub mod boxed;
pub mod char;
pub mod character;
pub mod either;
pub mod not;
pub mod option;
pub mod tuple;
pub mod vec;
pub mod vec1;

pub trait Parse<T>: TryParse<T> {
    fn parse(input: T) -> (T, Self);
}

pub trait TryParse<T>: Sized {
    fn try_parse(input: T) -> Option<(T, Self)>;

    fn try_parse_value(input: T) -> Option<Self> {
        TryParse::try_parse(input).map(|(_, value)| value)
    }
}
