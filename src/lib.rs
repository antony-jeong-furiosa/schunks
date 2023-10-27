#![crate_name="schunks"]

pub mod structs {
    pub use crate::{Schunks, Schunk};
}
pub use crate::structs::*;

pub extern crate rayon;
pub use rayon::prelude::*;

use std::{collections::VecDeque, sync::Mutex};
use rayon::iter::IterBridge;

/// An iterator that yields `Schunk`s, the chunk iterators.
///
/// Iterator element type is `Schunk`.
///
/// See [`.par_schunks()`](crate::SchunksTools::par_schunks) for more information.
pub struct Schunks<I: Iterator> {
    inner: Mutex<I>,
    size: usize,
}

impl<I> Iterator for Schunks<I>
    where I: Iterator
{
    type Item = Schunk<I>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut v = VecDeque::new();
        let mut inner = self.inner.lock().unwrap();
        for _ in 0..self.size {
            if let Some(e) = inner.next() {
                v.push_back(e);
            } else {
                break;
            }
        }
        drop(inner);
        if v.is_empty() {
            return None
        } else {
            Some(Schunk {
                inner: v,
            })
        }
    }
}

/// An iterator for the elements in a single chunk.
///
/// Iterator element type is `I::Item`.
pub struct Schunk<I: Iterator> {
    inner: VecDeque<I::Item>,
}

impl<I> Iterator for Schunk<I>
    where I: Iterator
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

/// An [`Iterator`] blanket implementation that provides extra methods.
pub trait SchunksTools: Iterator {
    /// Returns a *iterator* over at most `size` elements of
    /// `self` at a time. The chunks do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `size`, the last chunk may be shorter than `size`.  All
    /// other chunks will have that exact length.
    ///
    /// Iterator element type is `Schunk`, each chunk's iterator.
    ///
    /// **Panics** if `size` is 0.
    /// 
    /// # Example
    ///
    /// ```
    /// use schunks::*;
    ///
    /// let data = vec![1, 1, 2, -2, 6, 0, 3, 1];
    /// //chunk size=3 |------->|-------->|--->|
    ///
    /// for chunk in data.into_iter().schunks(3) {
    ///     // Check that the sum of each chunk is 4.
    ///     assert_eq!(4, chunk.sum());
    /// }
    /// ```
    fn schunks(self, size: usize) -> Schunks<Self>
        where Self: Sized
    {
        assert!(size != 0);
        Schunks {
            inner: Mutex::new(self),
            size,
        }
    }

    /// Returns a *parallel iterator* over at most `size` elements of
    /// `self` at a time. The chunks do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `size`, the last chunk may be shorter than `size`.  All
    /// other chunks will have that exact length.
    /// 
    /// Iterator element type is `Schunk`, each chunk's iterator.
    /// 
    /// Chunking by `size` elements is done keeping the order of the
    /// original iterator. However, the resulting parallel iterator
    /// is not guaranteed to keep the order among the chunks
    /// themselves.
    ///
    /// # Example
    ///
    /// ```
    /// use schunks::*;
    /// 
    /// let data = vec![1, 1, 0, -2, 8, 0, 3, 1];
    /// //chunk size=3 |------->|-------->|--->|
    /// 
    /// let mut result = data.iter().par_schunks(3).map(Iterator::sum::<i32>).collect::<Vec<_>>();
    /// result.sort_unstable();
    /// 
    /// assert_eq!(result, [2, 4, 6]);
    /// ```
    fn par_schunks(self, size: usize) -> IterBridge<Schunks<Self>>
        where Self: Sized + Send, Self::Item: Send
    {
        self.schunks(size).par_bridge()
    }
}

impl<T: ?Sized> SchunksTools for T where T: Iterator { }
