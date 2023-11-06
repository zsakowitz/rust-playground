use super::{Parse, TryParse};

impl<A: TryParse<T>, T: Clone> Parse<T> for Vec<A> {
    fn parse(mut input: T) -> (T, Self) {
        let mut output = Vec::new();

        while let Some((new_input, value)) = TryParse::try_parse(input.clone()) {
            input = new_input;
            output.push(value);
        }

        (input, output)
    }
}

impl<A: TryParse<T>, T: Clone> TryParse<T> for Vec<A> {
    fn try_parse(input: T) -> Option<(T, Self)> {
        Self::parse(input).into()
    }
}
