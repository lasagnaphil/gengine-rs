use super::bit_array::{BitArray, BitArray64};
use super::comp_storage::{ComponentID, ComponentStorage};

use std;
use std::collections::HashMap;

type EntityID = u32;

struct World {
    // map ctid -> ComponentStorage
    components: Vec<ComponentStorage>,
    // map ctid -> (map EntityID -> ComponentID)
    component_map: Vec<HashMap<EntityID, ComponentID>>,
    // map EntityID -> comp. bit array
    component_bits: HashMap<EntityID, BitArray64>,
    
    entity_count: u32
}

pub fn get_ctid<T>() -> u16 { 0 }

impl World {
    pub fn new() -> Self {
        World {
            components: Vec::new(),
            component_map: Vec::new(),
            component_bits: HashMap::new(),
            entity_count: 0
        }
    }

    pub fn register_component<T>(&mut self) {
        let storage = ComponentStorage::new(get_ctid::<T>(), std::mem::size_of::<T>() as u32, 1);
        self.components.push(storage);
    }

    pub fn create_entity(&mut self) -> EntityID {
        self.entity_count += 1;
        self.entity_count
    }

    pub fn add_component<T>(&mut self, entity: EntityID, component: T) {
        let comp_id = self.components[get_ctid::<T>() as usize].insert(component);
        self.component_map[comp_id.ctid() as usize].insert(entity, comp_id);
        self.component_bits.get_mut(&entity).unwrap().set_true(get_ctid::<T>() as usize);
    }

    pub fn remove_component<T>(&mut self, entity: EntityID, component: T) {
        let comp_id = match self.component_map[get_ctid::<T>() as usize].remove(&entity) {
            Some(comp_id) => comp_id,
            None => { return; }
        };
        self.components[get_ctid::<T>() as usize].release(comp_id);
        self.component_bits.get_mut(&entity).unwrap().set_false(get_ctid::<T>() as usize);
    }

    pub fn get_component<T>(&self, entity: EntityID) -> Option<&T> {
        self.component_map[get_ctid::<T>() as usize].get(&entity).map(|id| {
            self.components[get_ctid::<T>() as usize].get(*id)
        })
    }

    pub fn get_component_mut<T>(&mut self, entity: EntityID) -> Option<&mut T> {
        let id = match self.component_map[get_ctid::<T>() as usize].get(&entity) {
            Some(id) => *id,
            None => { return None; }
        };
        Some(self.components[get_ctid::<T>() as usize].get_mut(id))
    }

    pub fn remove_entity(&mut self, entity: EntityID) {
        for ctid in self.component_bits[&entity].iter() {
            self.component_map[ctid].remove(&entity);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use std::mem;
    use std::collections::HashMap;
    use super::World;

    type TestComp1 = (i32, f64, String);

    #[derive(PartialEq, Debug)]
    struct TestComp2(i32);

    #[test]
    fn test_world_register_component() {
        let mut world = World::new();
        world.register_component::<TestComp1>();
        world.register_component::<TestComp2>();

    }

}
