use super::handle::{new_handle, Handle, Skip, Take};
use super::lookup::Lookup;

/// Describes the result of appending `T` to the borrow-bag. This is useful in specifying the
/// return type when creating/modifying a `BorrowBag` in a function.
///
/// ## Examples
///
/// ```rust
/// # use borrow_bag::{Append, BorrowBag, Handle};
/// #
/// type SingleItemBag<T> = BorrowBag<(T, ())>;
/// type SingleItemHandle<T> = Handle<T, <() as Append<T>>::Navigator>;
///
/// fn single_item_bag<T>(t: T) -> (SingleItemBag<T>, SingleItemHandle<T>) {
///     BorrowBag::new().add(t)
/// }
/// #
/// # let (bag, handle) = single_item_bag(1u8);
/// # assert_eq!(*bag.borrow(handle), 1);
/// ```
pub trait Append<T> {
    /// The resulting `BorrowBag` type parameter after adding an element of type `T`.
    type Output: PrefixedWith<Self> + Lookup<T, Self::Navigator>;

    /// A type describing how to borrow the `T` which is added.
    ///
    /// If the output type is `(X, (Y, (Z, ())))`, we're adding the `Z` and so our `Navigator` will
    /// be `(Skip, (Skip, Take))`
    type Navigator; // 这里的Navigator 可以是 Take 或者 (Skip, (Skip, Take))

    /// Append the element, returning a new collection and a handle to borrow the element back.
    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>);
}

impl<T, U, V: Append<T>> Append<T> for (U, V) {
    type Output = (U, V::Output);
    type Navigator = (Skip, V::Navigator);

    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>) {
        let (u, v) = self;
        ((u, v.append(t).0), new_handle())
    }
}

impl<T> Append<T> for () {
    // This is the end of the added elements. We insert our `T` before the end.
    type Output = (T, ());

    // We're adding our `T` here, so we take this element on navigation.
    type Navigator = Take;

    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>) {
        ((t, ()), new_handle())
    }
}

/// Provides proof that the existing list elements don't move, which guarantees that existing
/// `Handle` values continue to work.
pub trait PrefixedWith<T>
where
    T: ?Sized,
{
}

impl<U> PrefixedWith<()> for (U, ()) {}

impl<U, T, V> PrefixedWith<(U, T)> for (U, V) where V: PrefixedWith<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_test() {
        let (list, _): ((u8, ()), Handle<u8, Take>) = ().append(1u8);
        let (list, _) = list.append(2u8);
        let (list, _) = list.append(3u8);

        assert_eq!(list.0, 1);
        assert_eq!((list.1).0, 2);
        assert_eq!(((list.1).1).0, 3);
    }
}
