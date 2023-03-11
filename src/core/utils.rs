use generational_arena::{Arena, Index};

use std::{cmp::PartialEq, fmt, marker::PhantomData};

pub struct TypedIndex<T>(Index, PhantomData<T>);

impl<T> From<Index> for TypedIndex<T> {
    fn from(id: Index) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> fmt::Display for TypedIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> fmt::Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for TypedIndex<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for TypedIndex<T> {}

impl<T> PartialEq for TypedIndex<T> {
    fn eq(&self, other: &TypedIndex<T>) -> bool {
        self.0 == other.0
    }
}

pub struct TypedArena<T>(Arena<T>);

impl<T> TypedArena<T> {
    pub fn new() -> Self {
        Self(Arena::new())
    }

    pub fn insert(&mut self, value: T) -> TypedIndex<T> {
        self.0.insert(value).into()
    }

    pub fn get(&self, index: TypedIndex<T>) -> Option<&T> {
        self.0.get(index.0)
    }

    pub fn remove(&mut self, index: TypedIndex<T>) -> Option<T> {
        self.0.remove(index.0)
    }
}

impl<T> std::ops::Deref for TypedArena<T> {
    type Target = Arena<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for TypedArena<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct RangeCheck {}

impl RangeCheck {
    pub fn check<E, T: PartialOrd, C: std::ops::RangeBounds<T>>(
        range: C,
        value: &T,
        error: E,
    ) -> Result<(), E> {
        if range.contains(value) {
            Ok(())
        } else {
            Err(error)
        }
    }
}
