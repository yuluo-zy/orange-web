use crate::state::{State, StateData};

/// A trait for accessing data that is stored in `State`.
///
/// This provides the easier `T::try_borrow_from(&state)` API (for example), as an alternative to
/// `state.try_borrow::<T>()`.
pub trait FromState: StateData + Sized {

    fn try_borrow_from(state: &State) -> Option<&Self>;

    fn borrow_from(state: &State) -> &Self;

    fn try_borrow_mut_from(state: &mut State) -> Option<&mut Self>;

    fn borrow_mut_from(state: &mut State) -> &mut Self;

    fn try_take_from(state: &mut State) -> Option<Self>;

    fn take_from(state: &mut State) -> Self;
}

impl<T> FromState for T
where
    T: StateData,
{
    fn try_borrow_from(state: &State) -> Option<&Self> {
        state.try_borrow()
    }

    fn borrow_from(state: &State) -> &Self {
        state.borrow()
    }

    fn try_borrow_mut_from(state: &mut State) -> Option<&mut Self> {
        state.try_borrow_mut()
    }

    fn borrow_mut_from(state: &mut State) -> &mut Self {
        state.borrow_mut()
    }

    fn try_take_from(state: &mut State) -> Option<Self> {
        state.try_take()
    }

    fn take_from(state: &mut State) -> Self {
        state.take()
    }
}
