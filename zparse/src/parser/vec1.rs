use super::{Parse, TryParse};
use vec1::Vec1;

/// This is an infinite loop. It's implemented for consistency with `Vec<T>`, and it's implemented
/// thoroughly to prevent dropping the return value.
impl<A: Parse<T>, T: Clone> Parse<T> for Vec1<A> {
    fn parse(mut input: T) -> (T, Self) {
        let mut output = Vec::new();

        loop {
            let (new_input, value) = A::parse(input.clone());
            input = new_input;
            output.push(value);
        }
    }
}

impl<A: TryParse<T>, T: Clone> TryParse<T> for Vec1<A> {
    fn try_parse(mut input: T) -> Option<(T, Self)> {
        let mut output = Vec::new();

        while let Some((new_input, value)) = TryParse::try_parse(input.clone()) {
            input = new_input;
            output.push(value);
        }

        output.try_into().ok().map(|output| (input, output))
    }
}
