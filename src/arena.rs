use std::marker::PhantomData;

pub type Ap<T> = ArenaPointer<T>;

pub struct Arena<T>{
    items: Vec<T>
}

impl<T> Arena<T>{
    pub fn new() -> Self{
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize{
        self.items.len()
    }

    pub fn with_capacity(capacity: usize) -> Self{
        Self { items: Vec::with_capacity(capacity) }
    }

    pub fn get(&self, ap: &Ap<T>) -> &T{
        &self.items[ap.0]
    }

    pub fn get_mut(&mut self, ap: &Ap<T>) -> &mut T{
        &mut self.items[ap.0]
    }

    pub fn allocate(&mut self, item: T) -> Ap<T>{
        self.items.push(item);
        ArenaPointer(self.items.len() - 1, PhantomData::default())
    }
}

pub struct ArenaPointer<T>(usize, PhantomData<T>);

impl<T> Clone for ArenaPointer<T>{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData::default())
    }
}

impl<T> Copy for ArenaPointer<T>{}

impl<T> PartialEq for ArenaPointer<T>{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> ArenaPointer<T>{
    pub fn get<'a>(&self, arena: &'a Arena<T>) -> &'a T{
        arena.get(self)
    }

    pub fn get_mut<'a>(&self, arena: &'a mut Arena<T>) -> &'a mut T{
        arena.get_mut(self)
    }

    pub fn internal_index(&self) -> usize{
        self.0
    }
}