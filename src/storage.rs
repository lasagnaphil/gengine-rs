use std;
use std::mem;
use std::marker::PhantomData;
use std::collections::HashMap;

#[derive(Derivative, PartialEq)]
#[derivative(Copy(bound=""), Clone(bound=""))]
pub struct ResourceID<T> {
    index: u32,
    generation: u16,
    tid: u16,
    phantom: PhantomData<T>
}

struct ItemNode<T> {
    item: T,
    next_index: u32,
    generation: u16,
    free: bool,
    name: String
}

pub struct Storage<T> {
    nodes: Vec<ItemNode<T>>,
    size: u32,

    first_available: u32,
    name_mappings: HashMap<String, u32>
}

static EMPTY_NODE_STR: &'static str = "<empty>";

impl<T> Storage<T> {
    pub fn new(capacity: u32) -> Self {
        assert!(capacity > 0);
        let mut nodes = Vec::<ItemNode<T>>::with_capacity(capacity as usize);
        unsafe {
            for i in 0..capacity {
                let node = nodes.as_mut_ptr().offset(i as isize);
                std::ptr::write(node, ItemNode {
                    item: mem::uninitialized(),
                    next_index: (i + 1) as u32,
                    generation: 0,
                    free: true,
                    name: String::from(EMPTY_NODE_STR)
                });
            }
        }
        Storage {
            nodes: nodes,
            size: 0,
            first_available: 0,
            name_mappings: HashMap::new()
        }
    }

    fn expand(&mut self) {
        let capacity = self.capacity();
        let new_capacity = capacity * 2;
        let mut new_nodes = Vec::with_capacity(new_capacity as usize);
        unsafe {
            std::ptr::copy(self.nodes.as_ptr(), new_nodes.as_mut_ptr(), capacity as usize);
            for i in capacity..new_capacity {
                let node = new_nodes.as_mut_ptr().offset(i as isize);
                std::ptr::write(node, ItemNode {
                    item: mem::uninitialized(),
                    next_index: i + 1,
                    generation: 0,
                    free: true,
                    name: String::from(EMPTY_NODE_STR)
                });
            }
        }
        self.nodes = new_nodes;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn capacity(&self) -> u32 {
        self.nodes.capacity() as u32
    }

    pub fn insert(&mut self, name: &str, item: T) -> (&T, ResourceID<T>) {
        if self.first_available == self.capacity() {
            self.expand();
        }
        let new_index = self.first_available;
        
        unsafe {
            let node = self.get_node_mut(new_index);
            self.first_available = (*node).next_index;

            assert!((*node).free);

            std::ptr::write(node, ItemNode {
                item: item,
                next_index: (*node).next_index,
                generation: (*node).generation + 1,
                free: false,
                name: name.to_string()
            });

            self.name_mappings.insert(name.to_string(), new_index);

            let node_ref = ResourceID {
                index: new_index,
                generation: (*node).generation,
                tid: 0,
                phantom: PhantomData
            };

            self.size += 1;

            return (&(*node).item, node_ref);
        }
    }

    pub fn has(&self, item_ref: ResourceID<T>) -> bool {
        unsafe {
            let node = self.get_node(item_ref.index);
            return !(*node).free && (*node).generation == item_ref.generation;
        }
    }
    
    pub fn get(&self, item_ref: ResourceID<T>) -> &T {
        unsafe {
            let node = self.get_node(item_ref.index);
            assert!(!(*node).free);
            assert!((*node).generation == item_ref.generation);

            return &((*node).item);
        }
    }

    pub fn get_mut(&mut self, item_ref: ResourceID<T>) -> &mut T {
        unsafe {
            let mut node = self.get_node_mut(item_ref.index);
            assert!(!(*node).free);
            assert!((*node).generation == item_ref.generation);

            return &mut ((*node).item);
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.name_mappings.get(name).map(|index| {
            unsafe {
                let node = self.get_node(*index);
                assert!(!(*node).free);
                return &((*node).item);
            }
        })
    }

    pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut T> {
        let index = match self.name_mappings.get(name) {
                Some(index) => *index,
                None => { return None; }
        };
        unsafe {
            let mut node = self.get_node_mut(index);
            assert!(!(*node).free);
            return Some(&mut ((*node).item));
        }
    }

    pub fn get_ref_by_name(&self, name: &str) -> Option<ResourceID<T>> {
        self.name_mappings.get(name).map(|index| {
            unsafe {
                let node = self.get_node(*index);
                assert!(!(*node).free);
                return ResourceID {
                    index: *index,
                    generation: (*node).generation,
                    tid: 0,
                    phantom: PhantomData
                };
            }
        })

    }

    pub fn release(&mut self, item_ref: ResourceID<T>) {
        let name = unsafe {
            let node = self.get_node_mut(item_ref.index);
            assert!(!(*node).free);
            (*node).free = true;

            (*node).next_index = self.first_available;
            self.first_available = item_ref.index;

            &(*node).name
        };
        self.size -= 1;
        self.name_mappings.remove(name);
    }

    pub fn release_by_name(&mut self, name: &str) {
        // Trying to work around lexical lifetimes :(
        let index = match self.name_mappings.get(name) {
            Some(index) => { *index }
            None => { return; } // TODO: report error instead of failing silently...
        };

        let name = unsafe {
            let node = self.get_node_mut(index);
            assert!(!(*node).free);
            (*node).free = true;

            (*node).next_index = self.first_available;
            self.first_available = index;

            &(*node).name
        };
        self.size -= 1;
        self.name_mappings.remove(name);
    }

    pub fn iterate<F>(&self, fun: F) where F : Fn(&T) -> () {
        for i in 0..self.capacity() {
            unsafe {
                let node = self.get_node(i);
                if !(*node).free {
                    fun(&(*node).item);
                }
            }
        }
    }
    
    // Helper methods to get ith node pointer
    
    unsafe fn get_node(&self, index: u32) -> *const ItemNode<T> {
        self.nodes.as_ptr().offset(index as isize)
    }

    unsafe fn get_node_mut(&mut self, index: u32) -> *mut ItemNode<T> {
        self.nodes.as_mut_ptr().offset(index as isize)
    }
}

impl<T> Drop for Storage<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.capacity() {
                let node = self.get_node_mut(i);
                if !(*node).free {
                    std::ptr::drop_in_place(node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use storage::*;
    use self::rand::Rng;

    type TestData1 = (i32, f64, String);

    #[derive(Debug)]
    struct TestData2(i32);

    #[test]
    fn test_storage_new() {
        let mut storage = Storage::<TestData1>::new(8);
        assert_eq!(storage.capacity(), 8);
    }

    #[test]
    fn test_storage_insert() {
        let mut storage = Storage::new(8);
        
        let (alice, alice_ref) = storage.insert("alice", (1, 3.0, "Alice".to_string()));
        assert_eq!(*alice, (1, 3.0, "Alice".to_string()));
    }

    #[test]
    fn test_storage_get() {
        let mut storage = Storage::new(8);
        
        let alice_ref = {
            let (alice, alice_ref) = storage.insert("alice", (1, 3.0, "Alice".to_string()));
            alice_ref
        };

        assert_eq!(*storage.get(alice_ref), (1, 3.0, "Alice".to_string()));
        assert_eq!(*storage.get_by_name("alice").unwrap(), (1, 3.0, "Alice".to_string()));
        {
            let temp_ref = storage.get_ref_by_name("alice").unwrap();
            assert_eq!(temp_ref.index, alice_ref.index);
            assert_eq!(temp_ref.generation, alice_ref.generation);
            assert_eq!(temp_ref.tid, alice_ref.tid);
        }
        assert!(storage.get_by_name("bob").is_none());
        assert!(storage.get_ref_by_name("bob").is_none());
    }

    #[test]
    #[should_panic]
    fn test_storage_get_panic() {
        let mut storage = Storage::new(8);

        let alice_ref = {
            let (alice, alice_ref) = storage.insert("alice", (1, 3.0, "Alice".to_string()));
            alice_ref
        };

        let invalid_ref = Ref::<TestData1> {
            index: 3,
            generation: 2,
            tid: 0,
            phantom: PhantomData
        };

        let _ = storage.get(invalid_ref);
    }

    #[test]
    fn test_storage_has() {
        let mut storage = Storage::new(8);
        
        let alice_ref = {
            let (alice, alice_ref) = storage.insert("alice", (1, 3.0, "Alice".to_string()));
            alice_ref
        };

        assert!(storage.has(alice_ref));
    }

    #[test]
    fn test_storage_size() {
        let mut storage = Storage::new(8);

        assert_eq!(storage.size(), 0);
        storage.insert("alice", (1, 1.0, "Alice".to_string()));
        assert_eq!(storage.size(), 1);
        storage.insert("bob", (2, 2.0, "Bob".to_string()));
        assert_eq!(storage.size(), 2);
        storage.insert("chris", (3, 3.0, "Chris".to_string()));
        assert_eq!(storage.size(), 3);
    }

    #[test]
    fn test_storage_release() {
        let mut storage = Storage::<TestData1>::new(8);

        let alice_ref = {
            let (alice, alice_ref) = storage.insert("alice", (1, 3.0, "Alice".to_string()));
            alice_ref
        };

        let bob_ref = {
            let (bob, bob_ref) = storage.insert("bob", (2, 4.0, "Bob".to_string()));
            bob_ref
        };

        storage.release(alice_ref);
        assert_eq!(storage.get_by_name("alice"), None);
        storage.release(bob_ref);
        assert_eq!(storage.get_by_name("bob"), None);
    }

    #[test]
    fn test_storage_many() {
        let test_size: u32 = 2048;
        let mut storage = Storage::new(test_size);
        for i in 0..test_size {
            let num_str = i.to_string();
            storage.insert(&num_str, TestData2(i as i32));
        }
        assert_eq!(storage.size(), test_size);

        let mut rng = rand::thread_rng();
        let mut vec = (0..1000).into_iter().map(|x| rng.gen::<u32>() % 2048).collect::<Vec<_>>();
        vec.sort();
        vec.dedup();

        for i in &vec {
            storage.release_by_name(&i.to_string());
        }
        let removed_size = vec.len() as u32;
        assert_eq!(storage.size(), test_size - removed_size);

        let mut count = 0;
        for i in &vec {
            count += 1;
            storage.insert(&i.to_string(), TestData2(*i as i32));
        }
        assert_eq!(storage.size(), test_size);
    }

    #[test]
    fn test_storage_expand() {
        let test_size: u32 = 65536;
        let mut storage = Storage::new(1);
        for i in 0..test_size {
            let num_str = i.to_string();
            storage.insert(&num_str, TestData2(i as i32));
        }
        assert_eq!(storage.size(), test_size);

        let mut rng = rand::thread_rng();
        let mut vec = (0..test_size).into_iter().map(|x| rng.gen::<u32>() % test_size).collect::<Vec<_>>();
        vec.sort();
        vec.dedup();

        for i in &vec {
            println!("Releasing item {}", i);
            storage.release_by_name(&i.to_string());
        }
        let removed_size = vec.len() as u32;
        assert_eq!(storage.size(), test_size - removed_size);

        let mut count = 0;
        for i in &vec {
            count += 1;
            storage.insert(&i.to_string(), TestData2(*i as i32));
        }
        assert_eq!(storage.size(), test_size);
    }
}
