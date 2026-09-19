use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::assets::{AssetModel, AssetModelSet};
use crate::game::Game;
use crate::graphics::{Gpu, ModelSet};

#[derive(Debug)]
pub struct Handle<T> {
    index: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Handle<T> {
    fn new(index: u32) -> Self {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct Store<T> {
    slots: Vec<Option<T>>,
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
        if let Some(free) = self.free.pop() {
            self.slots[free as usize] = Some(data);
            return Handle::new(free);
        }
        let id = self.slots.len();
        self.slots.push(Some(data));
        return Handle::new(id as u32);
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.slots[handle.index as usize].as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.slots[handle.index as usize].as_mut()
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        self.slots[handle.index as usize] = None;
        self.free.push(handle.index);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
        self.slots.iter()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
}

pub struct Engine {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub models: ModelSet,
}

impl Engine {
    pub fn new(window: Window, assets: &[AssetModel]) -> Self {
        let window = Arc::new(window);
        let mut gpu = Gpu::new(window.clone());
        gpu.resize(window.inner_size());

        let models = ModelSet::load(&gpu, &assets);

        Self {
            window,
            gpu,
            models,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }

    #[inline(always)]
    pub fn render(&mut self, game: &Game, assets: &AssetModelSet) {
        self.gpu.render(&self.window, game, &self.models, assets);
    }
}
