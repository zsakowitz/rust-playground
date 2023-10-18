use super::Parse;

impl<A: Parse<T>, T> Parse<T> for (A,) {
    type Error = A::Error;

    fn parse(input: T) -> Result<(T, Self), Self::Error> {
        let (input, a) = A::parse(input)?;
        Ok((input, (a,)))
    }
}

macro_rules! seq {
    ($first_upper:ident $first_lower:ident $($upper:ident $lower: ident)+) => {
        impl<$first_upper: Parse<T>, $($upper: Parse<T>,)+ T: Clone> Parse<T> for (A, $($upper),+)
        where
            $(<$first_upper as Parse<T>>::Error: From<<$upper as Parse<T>>::Error>),+
        {
            type Error = $first_upper::Error;

            fn parse(input: T) -> Result<(T, Self), Self::Error> {
                let (input, $first_lower) = $first_upper::parse(input)?;
                $(let (input, $lower) = $upper::parse(input)?;)+
                Ok((input, ($first_lower, $($lower),+)))
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
