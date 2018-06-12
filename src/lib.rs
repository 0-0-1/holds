// Copyright 2018 0-0-1 and Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This small library provides the [`Holds`][1] trait, which can be
//! implemented for a type that contains another value. This trait is meant for
//! "Range"-like types. Obviously Rust's `Range*` types fit this definition,
//! but slices like `[T]` and `str` do too (they contain a starting reference
//! and a length, and contain a number of subslices or discrete values in
//! between their start and end points).
//! 
//! [`Holds`][1] exposes the [`holds`][.1] method, which returns a `bool`
//! representing whether the value was held within. This library also provides
//! the [`Reassign`][2] trait, which is a subtrait of [`Holds`][1].
//! [`Reassign`][2] is intended soley for references, so that the lifetime of a
//! value reference can be extended if it's held by a container that lives
//! longer. [`reassign`][.2] therefore takes `&'a self` and `&'b T` and returns
//! [`Option`][1*] `<` `&'a T` `>`.
//! 
//! [1]: ./trait.Holds.html
//! [2]: ./trait.Reassign.html
//! 
//! [.1]: ./trait.Holds.html#tymethod.holds
//! [.2]: ./trait.Reassign.html#tymethod.reassign
//! 
//! [1*]: https://doc.rust-lang.org/std/option/enum.Option.html

#![cfg_attr(not(feature = "std"), no_std)]

#![cfg_attr(feature = "unstable", feature(collections_range))]

#[cfg(not(feature = "std"))]
mod imported {
    extern crate core;
    pub use core::ops::{
        Range,
        RangeFrom,
        RangeFull,
        RangeInclusive,
        RangeTo,
        RangeToInclusive
    };
    pub use core::slice;
    pub use core::str;

    pub use core::isize::MAX as ISIZE_MAX;

    #[cfg(feature = "unstable")]
    mod unstable {
        pub use core::ops::{
            Bound,
            RangeBounds
        };
    }

    #[cfg(feature = "unstable")]
    pub use self::unstable::*;
}

#[cfg(feature = "std")]
mod imported {
    pub use std::ops::{
        Range,
        RangeFrom,
        RangeFull,
        RangeInclusive,
        RangeTo,
        RangeToInclusive
    };
    pub use std::slice;
    pub use std::str;

    pub use std::isize::MAX as ISIZE_MAX;

    #[cfg(feature = "unstable")]
    mod unstable {
        pub use std::ops::{
            Bound,
            RangeBounds
        };
    }

    #[cfg(feature = "unstable")]
    pub use self::unstable::*;
}

use self::imported::*;

pub trait Holds<T> {
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool;
}

impl<T> Holds<T> for Range<T>
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool {
        *value >= self.start && *value < self.end
    }
}

impl<T> Holds<T> for RangeFrom<T>
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool {
        *value >= self.start
    }
}

impl<T> Holds<T> for RangeFull
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, _: &'b T) -> bool {
        true
    }
}

#[cfg(feature = "unstable")]
impl<T> Holds<T> for RangeInclusive<T>
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool {
        // Inclusive range fields are private on stable, and `RangeBounds` seems more
        // likely to stabilize than public inclusive fields.
        value >= self.start() && value <= self.end()
    }
}

impl<T> Holds<T> for RangeTo<T>
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool {
        *value < self.end
    }
}

#[cfg(feature = "unstable")]
impl<T> Holds<T> for RangeToInclusive<T>
where
    T: PartialOrd<T>
{
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b T) -> bool {
        // See comment on implementation for `RangeInclusive`.
        value <= self.end()
    }
}

impl<'c, T> Holds<&'c T> for [T] {
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b &'c T) -> bool {
        self.holds(&ref_to_slice(*value))
    }
}

impl<'c, T> Holds<&'c [T]> for [T] {
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b &'c [T]) -> bool {
        fn into_start_end<T>(slice: &[T]) -> (*const T, *const T) {
            let len = slice.len();
            let start = slice as *const _ as *const T;
            (start, unsafe {
                match len.checked_sub(ISIZE_MAX as usize) {
                    Some(len) => start.offset(ISIZE_MAX).offset(len as isize),
                    None => start.offset(len as isize)
                }
            })
        }

        let (start, end) = into_start_end(self);
        let (val_start, val_end) = into_start_end(*value);

        start >= val_start && start <= val_end && end <= val_end
    }
}

impl<'c> Holds<&'c str> for str {
    #[inline]
    fn holds<'a, 'b>(&'a self, value: &'b &'c str) -> bool {
        self.as_bytes().holds(&value.as_bytes())
    }
}

pub trait Reassign<T>
where
    for<'c> Self: Holds<&'c T>,
    T: ?Sized
{
    fn reassign<'a, 'b>(&'a self, reference: &'b T) -> Option<&'a T>;
}

impl<T> Reassign<T> for [T] {
    #[inline]
    fn reassign<'a, 'b>(&'a self, reference: &'b T) -> Option<&'a T> {
        self.reassign(ref_to_slice(reference)).map(|x| &x[0])
    }
}

impl<T> Reassign<[T]> for [T] {
    #[inline]
    fn reassign<'a, 'b>(&'a self, reference: &'b [T]) -> Option<&'a [T]> {
        if self.holds(&reference) {
            unsafe {
                Some(&*(reference as *const _))
            }
        } else {
            None
        }
    }
}

impl Reassign<str> for str {
    #[inline]
    fn reassign<'a, 'b>(&'a self, reference: &'b str) -> Option<&'a str> {
        unsafe {
            self.as_bytes().reassign(reference.as_bytes()).map(|x| str::from_utf8_unchecked(x))
        }
    }
}

#[inline]
fn ref_to_slice<T>(reference: &T) -> &[T]
{
    unsafe {
        slice::from_raw_parts(reference, 1)
    }
}