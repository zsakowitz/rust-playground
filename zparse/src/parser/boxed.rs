use super::{Parse, TryParse};

impl<A: Parse<T>, T> Parse<T> for Box<A> {
    fn parse(input: T) -> (T, Self) {
        let (input, value) = Parse::parse(input);
        (input, Box::new(value))
    }
}

impl<A: TryParse<T>, T> TryParse<T> for Box<A> {
    fn try_parse(input: T) -> Option<(T, Self)> {
        TryParse::try_parse(input).map(|(input, value)| (input, Box::new(value)))
    }
}
