use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

///´Source´'s are sources for some type T. Taking from a source returns an optional.
/// While a ´Source´ has things it should return Some(T).
/// If the ´Source´ permanently runs out of things it should return None signaling to
/// the user of the source that they should move on to do other things.
pub trait Source<T> {
    fn take(&mut self) -> Option<T>;
}

impl<T> Source<T> for Vec<T> {
    fn take(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }
}

//TODO: figure out an implementation an efficient implementation where you don't have to reverse the string beforehand.
impl Source<char> for String {
    // fn take(&mut self) -> Option<char> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         Some(self.remove(0))
    //     }
    // }
    fn take(&mut self) -> Option<char> {
        self.pop()
    }
}

impl<T> Source<T> for VecDeque<T> {
    fn take(&mut self) -> Option<T> {
        self.pop_front()
    }
}

impl<T> Source<T> for Receiver<T> {
    fn take(&mut self) -> Option<T> {
        self.recv().ok()
    }
}

/// ´Sink´s are things take take in some type T. Generaly Sinks are used in tandem
/// with sources so that if something is put into a sink it should appear in a source somewhere.
pub trait Sink<T> {
    fn put(&mut self, thing: T);
}

impl<T> Sink<T> for Vec<T> {
    fn put(&mut self, thing: T) {
        self.push(thing);
    }
}

impl<T> Sink<T> for VecDeque<T> {
    fn put(&mut self, thing: T) {
        self.push_back(thing);
    }
}

impl<'a, T> Sink<T> for &'a mut VecDeque<T> {
    fn put(&mut self, thing: T) {
        self.push_back(thing);
    }
}

impl<T> Sink<T> for Sender<T> {
    fn put(&mut self, thing: T) {
        self.send(thing).unwrap();
    }
}

pub struct State<M, Sy>(pub fn(&mut M, Sy) -> State<M, Sy>);
impl<M, Sy> Deref for State<M, Sy> {
    type Target = fn(&mut M, Sy) -> State<M, Sy>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
