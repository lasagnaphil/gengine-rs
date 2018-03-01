mod bit_array;
mod comp_storage;

use self::bit_array::{BitArray, BitArray64};
use self::comp_storage::{ComponentID, ComponentStorage};

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

fn get_ctid<T>() -> u16 { 0 }

impl World {
    pub fn new() -> Self {
        World {
            components: Vec::new(),
            component_map: Vec::new(),
            component_bits: HashMap::new(),
            entity_count: 0
        }
    }

    fn register_component<T>(&mut self) {
        let storage = ComponentStorage::new(get_ctid::<T>(), std::mem::size_of::<T>() as u32, 1);
        self.components.push(storage);
    }

    fn create_entity(&mut self) -> EntityID {
        self.entity_count += 1;
        self.entity_count
    }

    fn add_component<T>(&mut self, entity: EntityID, component: T) {
        let comp_id = self.components[get_ctid::<T>() as usize].insert(component);
        self.component_map[comp_id.ctid() as usize].insert(entity, comp_id);
        self.component_bits.get_mut(&entity).unwrap().set_true(get_ctid::<T>() as usize);
    }

    fn remove_component<T>(&mut self, entity: EntityID, component: T) {
        let comp_id = match self.component_map[get_ctid::<T>() as usize].remove(&entity) {
            Some(comp_id) => comp_id,
            None => { return; }
        };
        self.components[get_ctid::<T>() as usize].release(comp_id);
        self.component_bits.get_mut(&entity).unwrap().set_false(get_ctid::<T>() as usize);
    }

    fn remove_entity(&mut self, entity: EntityID) {
        for ctid in self.component_bits[&entity].iter() {
            self.component_map[ctid].remove(&entity);
        }
    }
}
