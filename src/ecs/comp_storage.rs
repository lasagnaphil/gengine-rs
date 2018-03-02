use super::get_ctid;

use std;
use std::mem;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct ComponentID {
    index: u32,
    generation: u16,
    ctid: u16
}

impl ComponentID {
    pub fn index(&self) -> u32 { self.index }
    pub fn generation(&self) -> u16 { self.generation }
    pub fn ctid(&self) -> u16 { self.ctid }
}

#[derive(Clone)]
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
        for i in 0..capacity {
            indices.push(ComponentIndex {
                packed_index: i,
                next_index: i + 1,
                generation: 0,
                free: true
            });
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
        let mut new_data = Vec::with_capacity((self.ctsize * new_capacity) as usize);
        unsafe {
            std::ptr::copy(self.data.as_ptr(), new_data.as_mut_ptr(), (self.ctsize * capacity) as usize);
        }
        self.indices.reserve(capacity as usize);
        for i in capacity..new_capacity {
            self.indices.push(ComponentIndex {
                packed_index: i,
                next_index: i + 1,
                generation: 0,
                free: true
            });
        }
        self.data = new_data;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn capacity(&self) -> u32 {
        self.indices.capacity() as u32
    }

    pub fn insert<T>(&mut self, item: T) -> ComponentID {
        if self.first_available == self.capacity() {
            self.expand();
        }
        let (comp_id, packed_index) = {
            let comp_index = &mut self.indices[self.first_available as usize];
            assert!(comp_index.free);

            let new_index = self.first_available;
            self.first_available = comp_index.next_index;

            comp_index.generation += 1;
            comp_index.free = false;

            self.size += 1;
            self.last_inserted = new_index;


            (ComponentID {
                index: new_index,
                generation: comp_index.generation,
                ctid: get_ctid::<T>()
            }, comp_index.packed_index)
        };
        unsafe {
            let item_loc = self.get_data_mut(packed_index);
            std::ptr::write(item_loc, item);
        }
        return comp_id;
    }

    pub fn has(&self, id: ComponentID) -> bool {
        let comp_index = &self.indices[id.index as usize];
        return !comp_index.free && comp_index.generation == id.generation;
    }

    pub fn get<T>(&self, id: ComponentID) -> &T {
        let comp_index = &self.indices[id.index as usize];

        assert!(!comp_index.free);
        assert!(comp_index.generation == id.generation);

        unsafe { &(*self.get_data(comp_index.packed_index)) }
    }

    pub fn try_get<T>(&self, id: ComponentID) -> Option<&T> {
        let comp_index = &self.indices[id.index as usize];

        if !comp_index.free && comp_index.generation == id.generation {
            unsafe { Some(&(*self.get_data(comp_index.packed_index))) }
        }
        else {
            None
        }
    }

    pub fn get_mut<T>(&mut self, id: ComponentID) -> &mut T {
        let comp_index = self.indices[id.index as usize].clone();

        assert!(!comp_index.free);
        assert!(comp_index.generation == id.generation);

        unsafe {
            &mut (*self.get_data_mut(comp_index.packed_index))
        }
    }

    pub fn try_get_mut<T>(&mut self, id: ComponentID) -> Option<&mut T> {
        let comp_index = self.indices[id.index as usize].clone();
        if !comp_index.free && comp_index.generation == id.generation {
            unsafe { Some(&mut (*self.get_data_mut(comp_index.packed_index))) }
        }
        else {
            None
        }
    }

    pub fn release(&mut self, id: ComponentID) {
        unsafe {
            let last_inserted = self.last_inserted;
            let (past_index, future_index) = {
                let past_index = {
                    let comp_index = &mut self.indices[id.index as usize];

                    assert!(!comp_index.free);
                    assert!(comp_index.generation == id.generation);
                    comp_index.free = true;
                    comp_index.packed_index
                };

                let future_index = self.indices[last_inserted as usize].packed_index;
                std::ptr::copy(
                    self.data.as_ptr().offset((self.ctsize * future_index) as isize),
                    self.data.as_mut_ptr().offset((self.ctsize * past_index) as isize),
                    self.ctsize as usize
                );
                (past_index, future_index)
            };
            self.indices[id.index as usize].packed_index = future_index;
            self.indices[last_inserted as usize].packed_index = past_index;

            self.indices[id.index as usize].next_index = self.first_available;
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
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        unsafe {
            // TODO
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use std::mem;
    use std::collections::HashMap;
    use super::ComponentStorage;
    use super::ComponentID;
    use self::rand::Rng;

    type TestData1 = (i32, f64, String);

    #[derive(PartialEq, Debug)]
    struct TestData2(i32);

    #[test]
    fn test_comp_storage_new() {
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData1>() as u32, 8);
        assert_eq!(storage.capacity(), 8);
    }

    #[test]
    fn test_comp_storage_insert_get() {
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData1>() as u32, 8);

        let alice_id = storage.insert((1, 3.0, "Alice".to_string()));
        assert!(storage.has(alice_id));
        {
            let alice : &TestData1 = storage.get(alice_id);
            assert_eq!(*alice, (1, 3.0, "Alice".to_string()));
        }
        let alice : &mut TestData1 = storage.get_mut(alice_id);
        alice.0 = 2;
        alice.2 = "Ashley".to_string();
        assert_eq!(*alice, (2, 3.0, "Ashley".to_string()));
    }

    #[test]
    #[should_panic]
    fn test_comp_storage_get_panic() {
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData1>() as u32, 8);

        let mut alice_id = storage.insert((1, 3.0, "Alice".to_string()));
        alice_id.index = 1;
        storage.get::<TestData1>(alice_id);
    }

    #[test]
    fn test_comp_storage_size() {
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData1>() as u32, 8);

        assert_eq!(storage.size(), 0);
        storage.insert((1, 1.0, "Alice".to_string()));
        assert_eq!(storage.size(), 1);
        storage.insert((2, 2.0, "Bob".to_string()));
        assert_eq!(storage.size(), 2);
        storage.insert((3, 3.0, "Chris".to_string()));
        assert_eq!(storage.size(), 3);
    }

    #[test]
    fn test_comp_storage_release() {
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData1>() as u32, 8);

        let alice_id = storage.insert((1, 3.0, "Alice".to_string()));
        let bob_id = storage.insert((2, 4.0, "Bob".to_string()));
        assert_eq!(storage.size(), 2);

        storage.release(alice_id);
        assert_eq!(storage.try_get::<TestData1>(alice_id), None);
        assert_eq!(*storage.try_get::<TestData1>(bob_id).unwrap(), (2, 4.0, "Bob".to_string()));
        assert_eq!(storage.size(), 1);

        storage.release(bob_id);
        assert_eq!(storage.try_get::<TestData1>(alice_id), None);
        assert_eq!(storage.try_get::<TestData1>(bob_id), None);
        assert_eq!(storage.size(), 0);
    }

    #[test]
    fn test_comp_storage_many() {
        let test_size: u32 = 2048;
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData2>() as u32, test_size);
        let mut ids: Vec<ComponentID> = Vec::new();
        for i in 0..test_size {
            let id = storage.insert(TestData2(i as i32));
            ids.push(id);
        }
        assert_eq!(storage.size(), test_size);

        let mut rng = rand::thread_rng();
        let mut vec = (0..1000).into_iter().map(|x| rng.gen::<u32>() % test_size).collect::<Vec<_>>();
        vec.sort();
        vec.dedup();

        for i in &vec {
            storage.release(ids[*i as usize]);
        }
        let removed_size = vec.len() as u32;
        assert_eq!(storage.size(), test_size - removed_size);
        for i in &vec {
            assert_eq!(storage.try_get::<TestData2>(ids[*i as usize]), None);
        }

        let mut count = 0;
        let mut new_ids: HashMap<ComponentID, TestData2> = HashMap::new();
        for i in &vec {
            count += 1;
            let id = storage.insert(TestData2(*i as i32));
            new_ids.insert(id, TestData2(*i as i32));
        }
        assert_eq!(storage.size(), test_size);
        for (id, value) in &new_ids {
            assert_eq!(*storage.try_get::<TestData2>(*id).unwrap(), *value);
        }
    }

    #[test]
    fn test_comp_storage_expand() {
        let test_size: u32 = 65536;
        let mut storage = ComponentStorage::new(0, mem::size_of::<TestData2>() as u32, 1);
        let mut ids: Vec<ComponentID> = Vec::new();
        for i in 0..test_size {
            let id = storage.insert(TestData2(i as i32));
            ids.push(id);
        }
        assert_eq!(storage.size(), test_size);

        let mut rng = rand::thread_rng();
        let mut vec = (0..test_size).into_iter().map(|x| rng.gen::<u32>() % test_size).collect::<Vec<_>>();
        vec.sort();
        vec.dedup();

        for i in &vec {
            // println!("Releasing {}", *i);
            // println!("Generation: {}", ids[*i as usize].generation());
            storage.release(ids[*i as usize]);
        }
        let removed_size = vec.len() as u32;
        assert_eq!(storage.size(), test_size - removed_size);
        for i in &vec {
            assert_eq!(storage.try_get::<TestData2>(ids[*i as usize]), None);
        }

        let mut count = 0;
        let mut new_ids: HashMap<ComponentID, TestData2> = HashMap::new();
        for i in &vec {
            count += 1;
            //println!("Inserting back {}", *i);
            let id = storage.insert(TestData2(*i as i32));
            new_ids.insert(id, TestData2(*i as i32));
        }
        assert_eq!(storage.size(), test_size);
        for (id, value) in &new_ids {
            //println!("checking id with value {:?}", *value);
            assert_eq!(*storage.try_get::<TestData2>(*id).unwrap(), *value);
        }
    }
}
