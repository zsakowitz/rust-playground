use super::Parse;
use std::convert::Infallible;

impl<A: Parse<T>, T: Clone> Parse<T> for Option<A> {
    type Error = Infallible;

    fn parse(input: T) -> Result<(T, Self), Self::Error> {
        match Parse::parse(input.clone()) {
            Ok((output, value)) => Ok((output.into(), Some(value))),
            Err(_) => Ok((input, None)),
        }
    }
}
