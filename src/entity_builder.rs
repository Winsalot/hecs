use std::alloc::{alloc, Layout};
use std::any::TypeId;
use std::mem::{self, MaybeUninit};
use std::ptr;

use crate::archetype::{Archetype, TypeInfo};
use crate::{Component, DynamicBundle};

/// Helper for incrementally constructing an entity with dynamic component types
///
/// Can be reused efficiently.
///
/// ```
/// # use hecs::*;
/// let mut world = World::new();
/// let mut builder = EntityBuilder::new();
/// builder.add(123).add("abc");
/// let e = world.spawn(builder.build());
/// assert_eq!(*world.get::<i32>(e).unwrap(), 123);
/// assert_eq!(*world.get::<&str>(e).unwrap(), "abc");
/// ```
pub struct EntityBuilder {
    storage: Box<[MaybeUninit<u8>]>,
    // Backwards from the end!
    cursor: *mut u8,
    max_align: usize,
    info: Vec<(TypeInfo, *mut u8)>,
    ids: Vec<TypeId>,
}

impl EntityBuilder {
    /// Create a builder representing an entity with no components
    pub fn new() -> Self {
        Self {
            storage: Box::new([]),
            cursor: ptr::null_mut(),
            max_align: 16,
            info: Vec::new(),
            ids: Vec::new(),
        }
    }

    /// Add `component` to the entity
    pub fn add<T: Component>(&mut self, component: T) -> &mut Self {
        self.max_align = self.max_align.max(mem::align_of::<T>());
        if (self.cursor as usize) < mem::size_of::<T>() {
            self.grow(mem::size_of::<T>());
        }
        unsafe {
            self.cursor = (self.cursor.sub(mem::size_of::<T>()) as usize
                & !(mem::align_of::<T>() - 1)) as *mut u8;
            if self.cursor.cast() < self.storage.as_mut_ptr() {
                self.grow(mem::size_of::<T>());
                self.cursor = (self.cursor.sub(mem::size_of::<T>()) as usize
                    & !(mem::align_of::<T>() - 1)) as *mut u8;
            }
            ptr::write(self.cursor.cast::<T>(), component);
        }
        self.info.push((TypeInfo::of::<T>(), self.cursor));
        self
    }

    fn grow(&mut self, min_increment: usize) {
        let new_len = (self.storage.len() + min_increment)
            .next_power_of_two()
            .max(self.storage.len() * 2)
            .max(64);
        unsafe {
            let alloc = alloc(Layout::from_size_align(new_len, self.max_align).unwrap())
                .cast::<MaybeUninit<u8>>();
            let mut new_storage = Box::from_raw(std::slice::from_raw_parts_mut(alloc, new_len));
            new_storage[new_len - self.storage.len()..].copy_from_slice(&self.storage);
            self.cursor = new_storage
                .as_mut_ptr()
                .add(new_len - self.storage.len())
                .cast();
            self.storage = new_storage;
        }
    }

    /// Construct a `Bundle` suitable for spawning
    pub fn build(&mut self) -> BuiltEntity<'_> {
        self.info.sort_unstable_by(|x, y| x.0.cmp(&y.0));
        self.ids.clear();
        self.ids.extend(self.info.iter().map(|x| x.0.id()));
        BuiltEntity { builder: self }
    }
}

unsafe impl Send for EntityBuilder {}
unsafe impl Sync for EntityBuilder {}

/// The output of an `EntityBuilder`, suitable for passing to `World::spawn`
pub struct BuiltEntity<'a> {
    builder: &'a mut EntityBuilder,
}

impl DynamicBundle for BuiltEntity<'_> {
    fn with_ids<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T {
        f(&self.builder.ids)
    }

    #[doc(hidden)]
    fn type_info(&self) -> Vec<TypeInfo> {
        self.builder.info.iter().map(|x| x.0).collect()
    }

    unsafe fn store(self, archetype: &mut Archetype, index: u32) {
        for (ty, component) in self.builder.info.drain(..) {
            archetype.put_dynamic(component, ty.id(), ty.layout(), index);
        }
    }
}

impl Drop for BuiltEntity<'_> {
    fn drop(&mut self) {
        for (ty, component) in self.builder.info.drain(..) {
            unsafe {
                ty.drop(component);
            }
        }
    }
}