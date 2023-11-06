use super::{Parse, TryParse};

impl<A: TryParse<T>, T: Clone> Parse<T> for Option<A> {
    fn parse(input: T) -> (T, Self) {
        A::try_parse(input.clone())
            .map(|(input, value)| (input, Some(value)))
            .unwrap_or((input, None))
    }
}

impl<A: TryParse<T>, T: Clone> TryParse<T> for Option<A> {
    fn try_parse(input: T) -> Option<(T, Self)> {
        Self::parse(input).into()
    }
}
