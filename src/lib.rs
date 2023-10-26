#![crate_name="schunks"]

pub mod structs {
    pub use crate::{Schunks, Schunk};
}
pub use crate::structs::*;

pub extern crate rayon;
pub use rayon::prelude::*;

use std::{collections::VecDeque, sync::Mutex};
use rayon::iter::IterBridge;

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

pub trait SchunksTools: Iterator {
    fn par_schunks(self, size: usize) -> IterBridge<Schunks<Self>>
        where Self: Sized + Send, Self::Item: Send
    {
        assert!(size != 0);
        let sc = Schunks {
            inner: Mutex::new(self),
            size,
        };
        sc.par_bridge()
    }
}

impl<T: ?Sized> SchunksTools for T where T: Iterator { }
