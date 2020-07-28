//! Different ways to butcher a field.
//!
//! This module defines the behaviour of each butchering method, and how they
//! interact with the butcher derive macro.
//!
//! This module contains multiple butchering methods, represented by structs.
//! These butchering methods implement the [`ButcheringMethod`] trait.
//! This trait gives a definition of how the output data must be generated
//! based on the input (what output type, what to do when input is borrowed and
//! what to do when input is owned).
//!
//! The [`ButcherField`] trait is implemented for every structure associated to
//! a field of struct or enum on which `Butcher` is derived.
//!
//! [`ButcheringMethod`]: trait.ButcheringMethod.html
//! [`ButcherField`]: trait.ButcherField.html

use std::borrow::{Borrow, Cow};
use std::ops::Deref;

use crate::Butcher;

/// Allow to unify the behavior of the different butchering methods.
///
/// `T` is the input type, which can be either owned or borrowed for `'cow`. The
/// `from_owned` and `from_borrowed` take either an owned or a borrowed `T`, and
/// produce a given output type.
pub trait ButcheringMethod<'cow, T>
where
    T: 'cow,
{
    /// The output type.
    type Output: 'cow;

    /// Create an output with an owned input.
    fn from_owned(i: T) -> Self::Output;

    /// Creates an output with a borrowed input.
    fn from_borrowed(i: &'cow T) -> Self::Output;
}

/// The regular method.
///
/// This method will transform a type `T` into a `Cow<T>`. It requires that `T`
/// is [`Clone`], but won't clone it.
///
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub struct Regular;

impl<'cow, T> ButcheringMethod<'cow, T> for Regular
where
    T: Clone + 'cow,
{
    type Output = Cow<'cow, T>;

    /// Create an `Owned` variant, containing `T`.
    fn from_owned(i: T) -> Self::Output {
        Cow::Owned(i)
    }

    /// Create a `Borrowed` variant, containing a reference to `T`.
    fn from_borrowed(i: &'cow T) -> Self::Output {
        Cow::Borrowed(i)
    }
}

/// The flatten method.
///
/// This method will transform a type `T` which implements [`Deref`] into a
/// `Cow<<T as Deref>::Target>`. This allows users not to have to deal with
/// for instance `Cow<String>`, and instead automatically use `Cow<str>`.
///
/// It requires `T` to implement [`Deref`] and `Borrow<<T as Deref>::Target>`,
/// `<T as Deref>::Target` to implement [`ToOwned`], and there must be
/// `<<T as Deref>::Target as ToOwned>::Owned = T`.
///
/// [`Deref`]: https://doc.rust-lang.org/std/ops/trait.Deref.html
/// [`ToOwned`]: https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html
pub struct Flatten;

impl<'cow, T> ButcheringMethod<'cow, T> for Flatten
where
    T: Deref + Borrow<<T as Deref>::Target> + 'cow,
    <T as Deref>::Target: ToOwned + 'cow,
    T: Into<<<T as Deref>::Target as ToOwned>::Owned>,
{
    type Output = Cow<'cow, <T as Deref>::Target>;

    /// Create an `Owned` variant, containing `T`.
    fn from_owned(i: T) -> Self::Output {
        Cow::Owned(i.into())
    }

    /// Create a `Borrowed` variant, containing a reference to `T`.
    fn from_borrowed(i: &'cow T) -> Self::Output {
        Cow::Borrowed(i)
    }
}

/// The unbox method.
///
/// This method allows to get rid of [`Box`] which is often used in order to
/// create recursive types.
///
/// It requires `T` to implement [`Clone`].
///
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub struct Unbox;

impl<'cow, T> ButcheringMethod<'cow, Box<T>> for Unbox
where
    T: Clone + 'cow,
{
    type Output = Cow<'cow, T>;

    /// Create an `Owned` variant, using the conversion requirements described
    /// previously.
    fn from_owned(i: Box<T>) -> Self::Output {
        Cow::Owned(*i)
    }

    /// Create a `Borrowed` variant, using the `Deref` trait.
    fn from_borrowed(i: &'cow Box<T>) -> Self::Output {
        Cow::Borrowed(Deref::deref(i))
    }
}

/// The copy method.
///
/// **Note**: this is not related to the `Copy` trait, but it effectively copies
/// some data.
///
/// This method does not output any [`Cow`] at all. Instead, it moves or copies
/// the data provided as input, using the [`Clone`] trait.
///
/// [`Cow`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub struct Copy;

impl<'cow, T> ButcheringMethod<'cow, T> for Copy
where
    T: Clone + 'cow,
{
    type Output = T;

    /// Move the data.
    ///
    /// This may be reduced to a no-op.
    fn from_owned(i: T) -> Self::Output {
        i
    }

    /// `Clone` the input data.
    fn from_borrowed(i: &'cow T) -> Self::Output {
        i.clone()
    }
}

/// The rebutcher method.
///
/// This method will butcher again the type which is marked as such.
///
/// # Example
///
/// In the following code, we destructure a struct inside another struct:
///
/// ```rust
/// use butcher::Butcher;
/// use std::borrow::Cow;
///
/// #[derive(Butcher, Clone)]
/// struct Foo {
///     #[butcher(rebutcher)]
///     bar: Bar,
/// }
/// #[derive(Butcher, Clone)]
/// struct Bar(usize);
///
/// let input = Foo { bar: Bar(42) };
/// let input = Cow::Borrowed(&input);
///
/// let ButcheredFoo { bar: ButcheredBar(value) } = Foo::butcher(input);
///
/// assert_eq!(value, Cow::Owned(42));
/// ```
///
/// It requires the type to implement `Butcher` and to implement [`ToOwned`]
/// such that `<T as ToOwned>::Owned = T`. The latter requirement can be
/// implemented with the [`Clone`] trait.
///
/// [`ToOwned`]: https://doc.rust-lang.org/std/borrow/trait.ToOwned.html
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
pub struct Rebutcher;

impl<'cow, T> ButcheringMethod<'cow, T> for Rebutcher
where
    T: Butcher<'cow> + ToOwned<Owned = T> + 'cow,
{
    type Output = <T as Butcher<'cow>>::Output;

    fn from_owned(i: T) -> Self::Output {
        <T as Butcher>::butcher(Cow::Owned(i))
    }

    fn from_borrowed(i: &'cow T) -> Self::Output {
        <T as Butcher>::butcher(Cow::Borrowed(i))
    }
}

/// Define the behaviour of a specific field of a struct or enum when it is
/// butchered.
///
/// Implementors just have to specify a correct butchering method. The rest is
/// automatically implemented.
pub trait ButcherField<'cow, T>
where
    T: 'cow,
{
    /// The method which will be used.
    type Method: ButcheringMethod<'cow, T>;

    fn from_owned(i: T) -> <Self::Method as ButcheringMethod<'cow, T>>::Output {
        <Self::Method as ButcheringMethod<'cow, T>>::from_owned(i)
    }

    fn from_borrowed(i: &'cow T) -> <Self::Method as ButcheringMethod<'cow, T>>::Output {
        <Self::Method as ButcheringMethod<'cow, T>>::from_borrowed(i)
    }
}

/*
use crate::Butcher;

#[derive(Butcher, Clone)]
struct Foo {
    #[butcher(rebutcher)]
    bar: Bar,
}
#[derive(Butcher, Clone)]
struct Bar(usize);
*/