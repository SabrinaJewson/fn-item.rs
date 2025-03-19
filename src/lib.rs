//! Helpers for working with closures that don’t capture any variables.
//!
//! The [`fn_item!`](crate::fn_item) macro makes a closure with no captures,
//! and can be accepted into functions with [`ImplFnItem!`].
//!
//! This is useful for dealing with function pointers in a more composable way.
//!
//! # Examples
//!
//! ```
//! fn make_fn_ptr<F>(_: ImplFnItem![F: for<'a> Fn(&'a str) -> u32]) -> fn(&str) -> u32
//! where
//!     F: for<'a> FnItem<(&'a str,), u32>
//! {
//!     |s| F::call((s,))
//! }
//!
//! let fn_ptr = make_fn_ptr(fn_item!(|s| s.parse::<u32>().unwrap()));
//! assert_eq!(fn_ptr("4115"), 4115);
//!
//! use fn_item::FnItem;
//! use fn_item::ImplFnItem;
//! use fn_item::fn_item;
//! ```
//!
//! If you don’t want to add a generic parameter to your outer function,
//! you can use an inner function instead:
//!
//! ```
//! fn make_fn_ptr((f, ..): ImplFnItem![for<'a> Fn(&'a str) -> u32]) -> fn(&str) -> u32 {
//!     fn inner<F: for<'a> FnItem<(&'a str,), u32>>(_: F) -> fn(&str) -> u32 {
//!         |s| F::call((s,))
//!     }
//!     inner(f)
//! }
//! # use fn_item::FnItem;
//! # use fn_item::ImplFnItem;
//! ```
#![no_std]
#![allow(clippy::items_after_test_module)]

/// Trait for closures that don’t capture any variables.
/// Generated by [`fn_item!`] expressions;
/// accepted into functions with [`ImplFnItem!`].
///
/// See [the crate root](crate) for examples.
pub trait FnItem<A, R = ()>: Send + Sync + Copy {
    /// Call the closure.
    fn call(args: A) -> R;
}

/// Make a closure with no captures.
/// Returns a value of type [`ImplFnItem!`].
///
/// # Examples
///
/// ```
/// fn f(_: ImplFnItem![Fn()]) {}
/// f(fn_item!(|| println!("Hello world!")));
/// # use fn_item::fn_item;
/// # use fn_item::ImplFnItem;
/// ```
///
/// Can be used with function items directly, too:
///
/// ```
/// # fn f(_: ImplFnItem![Fn()]) {}
/// fn my_function() {
///     println!("Hello world!");
/// }
/// f(fn_item!(my_function));
/// # use fn_item::fn_item;
/// # use fn_item::ImplFnItem;
/// ```
#[macro_export]
macro_rules! fn_item {
    ($e:expr $(,)?) => {
        match ::core::option::Option::None::<::core::convert::Infallible> {
            ::core::option::Option::None => (
                $crate::ඞ::assert_is_fn_item(),
                $crate::ඞ::coerce_to_fn_ptr($e),
                $e,
            ),
            // Use `e @ _` to make sure we don’t match a constant named `e`.
            #[expect(clippy::redundant_pattern)]
            ::core::option::Option::Some(e @ _) => {
                $crate::ඞ::fix_types(e, ::core::ptr::fn_addr_eq)
            }
        }
    };
}

/// The type of [`fn_item!`] expressions.
///
/// This is a tuple whose first element is the [`FnItem`] itself.
///
/// Note that explicit HRTBs (i.e. `for<…>`) will always be required
/// even in cases where the equivalent `Fn` would allow them to be elided.
///
/// # Examples
///
/// ```
/// fn make_fn_ptr<F>(_: ImplFnItem![F: for<'a> Fn(&'a str) -> u32]) -> fn(&str) -> u32
/// where
///     F: for<'a> FnItem<(&'a str,), u32>
/// {
///     |s| F::call((s,))
/// }
///
/// use fn_item::FnItem;
/// use fn_item::ImplFnItem;
/// ```
#[macro_export]
macro_rules! ImplFnItem {
    (
        $F:ty:
        $(for<$($l:lifetime),* $(,)?>)?
        Fn($($T:ty),* $(,)?) $(-> $R:ty)?
        $(,)?
    ) => {
        $crate::ImplFnItem<
            $F,
            $(for<$(#[allow(single_use_lifetimes)] $l,)*>)? fn($($T,)*) $(-> $R)?,
            impl ::core::marker::Send
                + ::core::marker::Sync
                + ::core::marker::Copy
                + $(for<$($l,)*>)? Fn($($T,)*) $(-> $R)?,
        >
    };

    (
        $(for<$($l:lifetime),* $(,)?>)?
        Fn($($T:ty),* $(,)?) $(-> $R:ty)?
        $(,)?
    ) => {
        $crate::ImplFnItem![
            impl $(for<$($l,)*>)? $crate::FnItem<($($T,)*) $(, $R)?>:
            $(for<$($l,)*>)? Fn($($T,)*) $(-> $R)?,
        ]
    };
}

mod private {
    /// The implementor of `FnItem`.
    ///
    /// Must be private to prevent `panic!()` being able to resolve to `impl FnItem`;
    /// can’t use `impl Trait`
    /// because HRTBs mean the trait solver needs to lazily resolve which traits are implemented.
    #[expect(missing_debug_implementations)]
    #[derive(Clone, Copy)]
    pub struct IsFnItem<P, F>(pub(super) PhantomData<(P, F)>);

    use core::marker::PhantomData;
}
use private::IsFnItem;

/// To pass one of these to a function, use the [`fn_item!`] macro.
///
/// This exists for documentation purposes only –
/// use the [`ImplFnItem!`] macro instead
/// when accepting a [`fn_item!`].
pub type ImplFnItem<FI, P, F> = (FI, PhantomData<P>, F);

/// Not public API. Used by the [`fn_item!`] macro.
#[doc(hidden)]
pub mod ඞ {
    /// This is **not** safe to call.
    /// In particular,
    /// - If `F` has captures that do not outlive its argument and return types,
    ///   this can be used to make function pointers
    ///   that have a longer lifetime than they should.
    /// - If `F` has uninhabited captures,
    ///   this can be used to generate uninhabited values in `call`.
    ///
    /// However, this is safe to use through the `fn_item!` macro
    /// since that ensures the closure has no captures.
    pub const fn assert_is_fn_item<P, F>() -> IsFnItem<P, F> {
        IsFnItem(PhantomData)
    }

    /// Called with a closure to coerce it to its function pointer type.
    pub const fn coerce_to_fn_ptr<P: Copy>(_: P) -> PhantomData<P> {
        PhantomData
    }

    /// For soundness, we must enforce that the resulting type of this macro is
    ///
    /// ```ignore
    /// (IsFnItem<P, F>, PhantomData<P>, F)
    /// ```
    ///
    /// _where the `F`s are the same_.
    ///
    /// Hence, we call this function in an unreachable match branch,
    /// enforcing that the `F` type is fixed.
    ///
    /// Additionally, we pass in `fn_addr_eq`,
    /// just to ensure `P` is a function pointer
    /// (since the `FnPtr` trait itself is unstable).
    pub const fn fix_types<P, F>(
        e: Infallible,
        _: fn(P, fn()) -> bool,
    ) -> (IsFnItem<P, F>, PhantomData<P>, F) {
        match e {}
    }

    use super::IsFnItem;
    use core::convert::Infallible;
    use core::marker::PhantomData;
}

macro_rules! impl_tuples {
    ($($($T:ident $t:ident)*,)*) => { $(
        impl<$($T,)* R, P, Func> FnItem<($($T,)*), R> for IsFnItem<P, Func>
        where
            // A bunch of traits only function pointers implement,
            // just to be sure we are working with a function pointer.
            // This is also checked perhaps more directly in the [`fn_item!`] macro,
            // which uses `core::ptr::fn_addr_eq` to ensure `P` is a function pointer.
            P: PartialEq + Eq + PartialOrd + Ord + Hash + Debug + fmt::Pointer,

            // We can’t just require `P = fn($($T,)*) → R`,
            // because there’s no way to be generic over all function pointers
            // in a way compatible with HRTBs.
            // For example, if `A` is a generic parameter,
            // `fn(&str)` is a distinct type from `fn(A)`.
            // So we just ensure that `P` and `Func` have the same signature when you call them.
            //
            // The `Send`, `Sync` and `Copy` bounds are there as
            // another last-resort check against misuse.
            P: Send + Sync + Copy + Fn($($T,)*) -> R,
            Func: Send + Sync + Copy + Fn($($T,)*) -> R,
        {
            fn call(($($t,)*): ($($T,)*)) -> R {
                // Last-resort checks against API misuse.
                // If these fail, something has gone seriously wrong
                // (or the user is using private API).
                const {
                    assert!(size_of::<Func>() == 0);
                    assert!(align_of::<Func>() == 1);
                }

                (unsafe { &*(&raw const *&()).cast::<Func>() })($($t,)*)
            }
        }
    )* };
}

impl_tuples! {
                     ,
                    A a,
                  A a B b,
                A a B b C c,
              A a B b C c D d,
            A a B b C c D d E e,
          A a B b C c D d E e F f,
        A a B b C c D d E e F f G g,
      A a B b C c D d E e F f G g H h,
    A a B b C c D d E e F f G g H h I i,
}

#[cfg(test)]
mod tests {
    #[expect(dead_code)]
    const fn const_fn(_: ImplFnItem![Fn()]) {}
    const _: () = const_fn(fn_item!(|| {}));
}

use core::fmt;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
