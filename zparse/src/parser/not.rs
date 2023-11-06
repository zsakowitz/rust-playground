use std::marker::PhantomData;

use super::TryParse;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct Not<T>(PhantomData<T>);

impl<A: TryParse<T>, T: Clone> TryParse<T> for Not<A> {
    fn try_parse(input: T) -> Option<(T, Self)> {
        match A::try_parse(input.clone()) {
            None => Some((input, Not(PhantomData))),
            Some(_) => None,
        }
    }
}
