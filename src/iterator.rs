//! An iterator over data wrapped in `Cow`

use std::borrow::Cow;
use std::marker::PhantomData;

enum CowIter<'a, I, Input, Iterr1, Iterr2>
where
    I: 'a + ToOwned,
    Iterr1: Iterator<Item = &'a I>,
    Iterr2: Iterator<Item = <I as ToOwned>::Owned>,
    Input: 'a + ToOwned,
    &'a Input: IntoIterator<Item = &'a I, IntoIter = Iterr1> + ToOwned,
    <Input as ToOwned>::Owned: IntoIterator<Item = <I as ToOwned>::Owned, IntoIter = Iterr2>,
{
    Borrowed(Iterr1, PhantomData<Input>),
    Owned(Iterr2, PhantomData<Input>),
}

impl<'a, I, Input, Iterr1, Iterr2> CowIter<'a, I, Input, Iterr1, Iterr2>
where
    I: 'a + ToOwned,
    Iterr1: Iterator<Item = &'a I>,
    Iterr2: Iterator<Item = <I as ToOwned>::Owned>,
    Input: 'a + ToOwned,
    &'a Input: IntoIterator<Item = &'a I, IntoIter = Iterr1>,
    <Input as ToOwned>::Owned: IntoIterator<Item = <I as ToOwned>::Owned, IntoIter = Iterr2>,
{
    fn from_cow(i: Cow<'a, Input>) -> CowIter<I, Input, Iterr1, Iterr2> {
        match i {
            Cow::Owned(i) => CowIter::Owned(i.into_iter(), PhantomData),
            Cow::Borrowed(i) => {
                let i: &'a Input = i;
                CowIter::Borrowed(i.into_iter(), PhantomData)
            }
        }
    }
}

impl<'a, I, Input, Iterr1, Iterr2> Iterator for CowIter<'a, I, Input, Iterr1, Iterr2>
where
    I: 'a + ToOwned,
    Iterr1: Iterator<Item = &'a I>,
    Iterr2: Iterator<Item = <I as ToOwned>::Owned>,
    Input: 'a + ToOwned,
    &'a Input: IntoIterator<Item = &'a I, IntoIter = Iterr1> + ToOwned,
    <Input as ToOwned>::Owned: IntoIterator<Item = <I as ToOwned>::Owned, IntoIter = Iterr2>,
{
    type Item = Cow<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CowIter::Borrowed(it, _) => it.next().map(Cow::Borrowed),
            CowIter::Owned(it, _) => it.next().map(Cow::Owned),
        }
    }
}

#[cfg(test)]
mod cow_iter {
    use super::*;

    #[test]
    fn from_cow() {
        let i = vec![4usize, 1, 3, 5];
        let mut i = CowIter::from_cow(Cow::Owned(i));
        assert_eq!(i.next(), Some(Cow::Owned(4)));
        assert_eq!(i.next(), Some(Cow::Owned(1)));
        assert_eq!(i.next(), Some(Cow::Owned(3)));
        assert_eq!(i.next(), Some(Cow::Owned(5)));
        assert_eq!(i.next(), None);
    }
}
