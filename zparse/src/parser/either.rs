use super::Parse;

enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: Parse<T>, B: Parse<T>, T: Clone> Parse<T> for Either<A, B> {
    type Error = &'static str;

    fn parse(input: T) -> Result<(T, Self), Self::Error> {
        if let Ok((input, a)) = A::parse(input.clone()) {
            Ok((input, Either::A(a)))
        } else if let Ok((input, b)) = B::parse(input) {
            Ok((input, Either::B(b)))
        } else {
            Err("neither alternative matched")
        }
    }
}
