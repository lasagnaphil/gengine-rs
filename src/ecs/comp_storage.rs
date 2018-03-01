use super::get_ctid;

use std;

#[derive(Copy, Clone)]
pub struct ComponentID {
    index: u32,
    generation: u16,
    ctid: u16
}

impl ComponentID {
    pub fn ctid(&self) -> u16 { self.ctid }
}

struct ComponentIndex {
    packed_index: u32,
    next_index: u32,
    generation: u16,
    free: bool
}

pub struct ComponentStorage {
    data: Vec<u8>,
    indices: Vec<ComponentIndex>,
    size: u32,
    
    first_available: u32,
    last_inserted: u32,

    ctid: u16,
    ctsize: u32,
}

impl ComponentStorage {
    pub fn new(ctid: u16, ctsize: u32, capacity: u32) -> Self {
        assert!(capacity > 0);
        let mut data = Vec::with_capacity((ctsize * capacity) as usize);
        let mut indices = Vec::<ComponentIndex>::with_capacity(capacity as usize);
        unsafe {
            for i in 0..capacity {
                let comp_index = indices.as_mut_ptr().offset(i as isize);
                std::ptr::write(comp_index, ComponentIndex {
                    packed_index: i,
                    next_index: i + 1,
                    generation: 0,
                    free: true
                });
            }
        }
        ComponentStorage {
            data: data,
            indices: indices,
            size: 0,
            first_available: capacity,
            last_inserted: 0,
            ctid: ctid,
            ctsize: ctsize,
        }
    }

    fn expand(&mut self) {
        let capacity = self.capacity();
        let new_capacity = capacity * 2;
        let mut new_data = Vec::with_capacity((self.ctsize * capacity) as usize);
        let mut new_indices = Vec::with_capacity(new_capacity as usize);
        unsafe {
            std::ptr::copy(self.data.as_ptr(), new_data.as_mut_ptr(), (self.ctsize * capacity) as usize);
            std::ptr::copy(self.indices.as_ptr(), new_indices.as_mut_ptr(), capacity as usize);
        }
            
        unsafe {
            for i in capacity..new_capacity {
                let comp_index = new_indices.as_mut_ptr().offset(i as isize);
                std::ptr::write(comp_index, ComponentIndex {
                    packed_index: i,
                    next_index: i + 1,
                    generation: 0,
                    free: true
                });
            }
        }
        self.data = new_data;
        self.indices = new_indices;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn capacity(&self) -> u32 {
        self.indices.capacity() as u32
    }

    pub fn insert<T>(&mut self, item: T) -> ComponentID {
        let first_available = self.first_available;
        if first_available == self.capacity() {
            self.expand();
        }

        unsafe {
            let comp_index = self.get_index_obj_mut(first_available);
            assert!((*comp_index).free);

            let new_index = self.first_available;
            self.first_available = (*comp_index).next_index;

            (*comp_index).generation += 1;
            (*comp_index).free = false;

            self.size += 1;
            self.last_inserted = new_index;
            let item_loc = self.get_data_mut((*comp_index).packed_index);
            std::ptr::write(item_loc, item);

            return ComponentID {
                index: new_index,
                generation: (*comp_index).generation,
                ctid: get_ctid::<T>()
            };
        }
    }

    pub fn has(&self, id: ComponentID) -> bool {
        unsafe {
            let comp_index = self.get_index_obj(id.index);
            return !(*comp_index).free && (*comp_index).generation == id.generation;
        }
    }

    pub fn get<T>(&self, id: ComponentID) -> &T {
        unsafe {
            let comp_index = self.get_index_obj(id.index);

            assert!(!(*comp_index).free);
            assert!((*comp_index).generation == id.generation);

            &(*self.get_data((*comp_index).packed_index))
        }
    }

    pub fn get_mut<T>(&mut self, id: ComponentID) -> &mut T {
        unsafe {
            let comp_index = self.get_index_obj_mut(id.index);

            assert!(!(*comp_index).free);
            assert!((*comp_index).generation == id.generation);

            &mut (*self.get_data_mut((*comp_index).packed_index))
        }
    }
    
    pub fn release(&mut self, id: ComponentID) {
        unsafe {
            let last_inserted = self.last_inserted;
            let comp_index = self.get_index_obj_mut(id.index);
            let last_inserted_comp_index = self.get_index_obj_mut(last_inserted);

            assert!(!(*comp_index).free);
            assert!((*comp_index).generation == id.generation);

            let past_index = (*comp_index).packed_index;
            let future_index = (*last_inserted_comp_index).packed_index;
            std::ptr::copy(
                self.data.as_ptr().offset(past_index as isize),
                self.data.as_mut_ptr().offset(future_index as isize),
                self.ctsize as usize
            );
            (*comp_index).packed_index = future_index;
            (*last_inserted_comp_index).packed_index = past_index;

            (*comp_index).next_index = self.first_available;
            self.first_available = id.index;
        }
        self.size -= 1;
    }

    pub fn iterate<T, F>(&self, fun: F) where F : Fn(&T) -> () {
        for i in 0..self.size() {
            unsafe {
                fun(&(*self.get_data::<T>(i)));
            }
        }
    }

    unsafe fn get_data<T>(&self, index: u32) -> *const T {
        self.data.as_ptr().offset((self.ctsize * index) as isize) as *const T
    }

    unsafe fn get_data_mut<T>(&mut self, index: u32) -> *mut T {
        self.data.as_mut_ptr().offset((self.ctsize * index) as isize) as *mut T
    }

    unsafe fn get_index_obj(&self, index: u32) -> *const ComponentIndex {
        self.indices.as_ptr().offset(index as isize) as *const ComponentIndex
    }

    unsafe fn get_index_obj_mut(&mut self, index: u32) -> *mut ComponentIndex {
        self.indices.as_mut_ptr().offset(index as isize) as *mut ComponentIndex
    }
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.size() {
                // TODO: std::ptr::drop_in_place
            }
        }
    }
}
