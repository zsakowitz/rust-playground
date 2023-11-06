use super::{Parse, TryParse};

impl<T> Parse<T> for () {
    fn parse(input: T) -> (T, Self) {
        (input, ())
    }
}

impl<T> TryParse<T> for () {
    fn try_parse(input: T) -> Option<(T, Self)> {
        (input, ()).into()
    }
}

impl<A: Parse<T>, T> Parse<T> for (A,) {
    fn parse(input: T) -> (T, Self) {
        let (input, a) = A::parse(input);
        (input, (a,))
    }
}

impl<A: TryParse<T>, T> TryParse<T> for (A,) {
    fn try_parse(input: T) -> Option<(T, Self)> {
        let (input, a) = A::try_parse(input)?;
        Some((input, (a,)))
    }
}

macro_rules! seq {
    ($first_upper:ident $first_lower:ident $($upper:ident $lower: ident)+) => {
        impl<$first_upper: Parse<T>, $($upper: Parse<T>,)+ T: Clone> Parse<T> for (A, $($upper),+)
        {
            fn parse(input: T) -> (T, Self) {
                let (input, $first_lower) = $first_upper::parse(input);
                $(let (input, $lower) = $upper::parse(input);)+
                (input, ($first_lower, $($lower),+))
            }
        }

        impl<$first_upper: TryParse<T>, $($upper: TryParse<T>,)+ T: Clone> TryParse<T> for (A, $($upper),+)
        {
            fn try_parse(input: T) -> Option<(T, Self)> {
                let (input, $first_lower) = $first_upper::try_parse(input)?;
                $(let (input, $lower) = $upper::try_parse(input)?;)+
                Some((input, ($first_lower, $($lower),+)))
            }
        }
    };
}

seq!(A a B b);
seq!(A a B b C c);
seq!(A a B b C c D d);
seq!(A a B b C c D d E e);
seq!(A a B b C c D d E e F f);
seq!(A a B b C c D d E e F f G g);
seq!(A a B b C c D d E e F f G g H h);
