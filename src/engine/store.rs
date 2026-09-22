#[derive(Debug)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Handle<T> {
    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }
}

pub struct Slot<T> {
    data: Option<T>,
    generation: u32,
}

pub struct Store<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Store<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn create(&mut self, data: T) -> Handle<T> {
        if let Some(index) = self.free.pop() {
            let slot = self.slots.get_mut(index as usize).unwrap();
            slot.data = Some(data);
            return Handle::new(index, slot.generation);
        } else {
            let id = self.slots.len();
            self.slots.push(Slot {
                data: Some(data),
                generation: 0,
            });
            return Handle::new(id as u32, 0);
        }
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        let slot = self.slots.get_mut(handle.index as usize).unwrap();
        assert_eq!(slot.generation, handle.generation, "stale handle");

        slot.generation = slot.generation.wrapping_add(1);
        slot.data = None;
        self.free.push(handle.index);
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index as usize).unwrap();
        assert_eq!(slot.generation, handle.generation, "stale handle");

        slot.data.as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index as usize).unwrap();
        assert_eq!(slot.generation, handle.generation, "stale handle");

        slot.data.as_mut()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
}
