mod archetype;

use std::any::TypeId;

use downcast_rs::{impl_downcast, Downcast};
use fxhash::FxHashMap;

use archetype::{Archetype, TypeInfo};

pub struct World {
    entities: Vec<EntityMeta>,
    free: Vec<u32>,
    archetypes: Vec<Archetype>,
    archetype_index: FxHashMap<Vec<TypeId>, usize>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free: Vec::new(),
            archetypes: Vec::new(),
            archetype_index: FxHashMap::default(),
        }
    }

    pub fn spawn(&mut self, components: impl ComponentSet) -> Entity {
        use std::collections::hash_map::Entry;

        let entity = match self.free.pop() {
            Some(i) => Entity {
                generation: self.entities[i as usize].generation,
                id: i,
            },
            None => {
                let i = self.entities.len() as u32;
                self.entities.push(EntityMeta {
                    generation: 0,
                    archetype: 0,
                    index: 0,
                });
                Entity {
                    generation: 0,
                    id: i,
                }
            }
        };
        let archetype = match self.archetype_index.entry(components.elements()) {
            Entry::Occupied(x) => *x.get(),
            Entry::Vacant(x) => {
                self.archetypes.push(Archetype::new(components.info()));
                let index = self.archetypes.len() - 1;
                x.insert(index);
                index
            }
        };
        let archetype = &mut self.archetypes[archetype];
        unsafe {
            self.entities[entity.id as usize].index = archetype.allocate(entity.id);
            archetype.store(components);
        }
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        let meta = &mut self.entities[entity.id as usize];
        if meta.generation != entity.generation {
            return false;
        }
        meta.generation += 1;
        unsafe {
            self.archetypes[meta.archetype as usize].remove(meta.index);
        }

        true
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let meta = &self.entities[entity.id as usize];
        if meta.generation != entity.generation {
            return None;
        }
        unsafe { Some(self.archetypes[meta.archetype as usize].get(meta.index)) }
    }

    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let meta = &self.entities[entity.id as usize];
        if meta.generation != entity.generation {
            return None;
        }
        unsafe { Some(self.archetypes[meta.archetype as usize].get_mut(meta.index)) }
    }
}

pub trait Component: Downcast + Send + Sync + 'static {}
impl_downcast!(Component);
impl<T: Send + Sync + 'static> Component for T {}

struct EntityMeta {
    generation: u32,
    archetype: u32,
    index: u32,
}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    generation: u32,
    id: u32,
}

pub trait ComponentSet {
    fn elements(&self) -> Vec<TypeId>;
    fn info(&self) -> Vec<TypeInfo>;
    unsafe fn store(self, base: *mut u8, offsets: &FxHashMap<TypeId, usize>);
}

macro_rules! tuple_impl {
    ($($name: ident),*) => {
        impl<$($name: Component),*> ComponentSet for ($($name,)*) {
            fn elements(&self) -> Vec<TypeId> {
                vec![$(TypeId::of::<$name>()),*]
            }
            fn info(&self) -> Vec<TypeInfo> {
                vec![$(TypeInfo::of::<$name>()),*]
            }
            unsafe fn store(self, base: *mut u8, offsets: &FxHashMap<TypeId, usize>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(
                    base.add(*offsets.get(&TypeId::of::<$name>()).unwrap())
                        .cast::<$name>()
                        .write($name);
                )*
            }
        }
    }
}

tuple_impl!(A);
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE, AF);
// tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE, AF, AG);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut world = World::new();
        let e = world.spawn(("hi", 42));
        assert_eq!(world.get::<&'static str>(e), Some(&"hi"));
        assert_eq!(world.get::<i32>(e), Some(&42));
    }
}