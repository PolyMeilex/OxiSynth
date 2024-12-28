use std::marker::PhantomData;

enum Slot<T> {
    Free,
    Occupied { generation: usize, value: T },
}

impl<T> Slot<T> {
    fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    fn is_occupied(&self) -> bool {
        !self.is_free()
    }
}

pub struct Index<T> {
    id: usize,
    generation: usize,
    _ph: PhantomData<fn() -> T>,
}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}

impl<T> Eq for Index<T> {}

impl<T> std::fmt::Debug for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("id", &self.id)
            .field("generation", &self.generation)
            .finish()
    }
}

impl<T> Copy for Index<T> {}
impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(crate) struct Arena<T> {
    slots: Vec<Slot<T>>,
    generation: usize,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            generation: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> Index<T> {
        let entry = self.slots.iter_mut().enumerate().find(|(_, e)| e.is_free());

        match entry {
            Some((id, entry)) => {
                *entry = Slot::Occupied {
                    generation: self.generation,
                    value,
                };

                Index {
                    id,
                    generation: self.generation,
                    _ph: PhantomData,
                }
            }
            None => {
                // No free slots found, insert new one

                let id = self.slots.len();

                self.slots.push(Slot::Occupied {
                    generation: self.generation,
                    value,
                });

                Index {
                    id,
                    generation: self.generation,
                    _ph: PhantomData,
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|v| v.is_occupied()).count()
    }

    pub fn get(&self, index: Index<T>) -> Option<&T> {
        match self.slots.get(index.id)? {
            Slot::Occupied { generation, value } if *generation == index.generation => Some(value),
            _ => None,
        }
    }

    pub fn remove(&mut self, index: Index<T>) -> Option<T> {
        match self.slots.get_mut(index.id)? {
            Slot::Occupied { generation, .. } if *generation == index.generation => {
                let Slot::Occupied { value, .. } =
                    std::mem::replace(&mut self.slots[index.id], Slot::Free)
                else {
                    unreachable!();
                };

                self.generation += 1;
                Some(value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut arena = Arena::new();

        let a = arena.insert(1);
        let b = arena.insert(2);
        let c = arena.insert(3);

        assert_eq!((a.generation, a.id), (0, 0));
        assert_eq!((b.generation, b.id), (0, 1));
        assert_eq!((c.generation, c.id), (0, 2));

        assert_eq!(arena.get(a), Some(&1));
        assert_eq!(arena.get(b), Some(&2));
        assert_eq!(arena.get(c), Some(&3));
    }

    #[test]
    fn remove() {
        let mut arena = Arena::new();

        let a = arena.insert(1);
        let b = arena.insert(2);
        let c = arena.insert(3);

        assert_eq!((a.generation, a.id), (0, 0));
        assert_eq!((b.generation, b.id), (0, 1));
        assert_eq!((c.generation, c.id), (0, 2));

        assert_eq!(arena.remove(a), Some(1));
        assert_eq!(arena.remove(b), Some(2));
        assert_eq!(arena.remove(c), Some(3));

        assert_eq!(arena.get(a), None);
        assert_eq!(arena.get(b), None);
        assert_eq!(arena.get(c), None);
    }

    #[test]
    fn remove_and_reinsert() {
        let mut arena = Arena::new();

        let a = arena.insert(1);
        let b = arena.insert(2);
        let c = arena.insert(3);

        assert_eq!((a.generation, a.id), (0, 0));
        assert_eq!((b.generation, b.id), (0, 1));
        assert_eq!((c.generation, c.id), (0, 2));

        assert_eq!(arena.remove(a), Some(1));
        assert_eq!(arena.remove(b), Some(2));
        assert_eq!(arena.remove(c), Some(3));

        let aa = arena.insert(1);
        let bb = arena.insert(2);
        let cc = arena.insert(3);

        assert_eq!((aa.generation, aa.id), (3, 0));
        assert_eq!((bb.generation, bb.id), (3, 1));
        assert_eq!((cc.generation, cc.id), (3, 2));

        assert_eq!(arena.get(a), None);
        assert_eq!(arena.get(b), None);
        assert_eq!(arena.get(c), None);

        assert_eq!(arena.get(aa), Some(&1));
        assert_eq!(arena.get(bb), Some(&2));
        assert_eq!(arena.get(cc), Some(&3));
    }
}
