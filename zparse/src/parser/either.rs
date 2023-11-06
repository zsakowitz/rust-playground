use super::{Parse, TryParse};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: TryParse<T>, B: Parse<T>, T: Clone> Parse<T> for Either<A, B> {
    fn parse(input: T) -> (T, Self) {
        if let Some((input, a)) = A::try_parse(input.clone()) {
            (input, Either::A(a))
        } else {
            let (input, b) = B::parse(input);
            (input, Either::B(b))
        }
    }
}

impl<A: TryParse<T>, B: TryParse<T>, T: Clone> TryParse<T> for Either<A, B> {
    fn try_parse(input: T) -> Option<(T, Self)> {
        if let Some((input, a)) = A::try_parse(input.clone()) {
            Some((input, Either::A(a)))
        } else if let Some((input, b)) = B::try_parse(input) {
            Some((input, Either::B(b)))
        } else {
            None
        }
    }
}
