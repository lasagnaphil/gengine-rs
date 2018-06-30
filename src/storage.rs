use std;
use std::mem;
use std::fmt;
use std::marker::PhantomData;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeSeq};
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess};

pub trait Resource {
    fn tid() -> u16;
}

#[derive(Derivative, PartialEq, Debug)]
#[derivative(Copy(bound=""), Clone(bound=""))]
#[repr(C)]
pub struct ResourceID<T: Resource> {
    index: u32,
    generation: u16,
    tid: u16,
    phantom: PhantomData<T>
}

impl<T> ResourceID<T> where T: Resource {
    #[inline]
    pub fn null() -> ResourceID<T> {
        ResourceID {
            index: u32::max_value(),
            generation: 0,
            tid: T::tid(),
            phantom: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.index == u32::max_value()
    }
}

impl<T> Default for ResourceID<T> where T: Resource {
    fn default() -> Self {
        ResourceID::<T>::null()
    }
}

impl<T> fmt::LowerHex for ResourceID<T> where T: Resource {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:x}", self)
    }
}

impl<T> Serialize for ResourceID<T> where T: Resource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self))
    }
}

impl<'de, T> Deserialize<'de> for ResourceID<T> where T: Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        use serde::de::Error;
        let s: &str = Deserialize::deserialize(deserializer)?;
        let data = u64::from_str_radix(&str::replace(&s[2..], "_", ""), 16).map_err(D::Error::custom)?;
        unsafe {
            let rid = mem::transmute::<u64, ResourceID<T>>(data);
            Ok(rid)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ItemNode<T: Resource> {
    item: Option<T>,
    next_index: u32,
    generation: u16,
    name: String
}

#[derive(Serialize, Deserialize)]
pub struct Storage<T: Resource> {
    nodes: Vec<ItemNode<T>>,
    size: u32,

    first_available: u32,
    name_mappings: HashMap<String, u32>
}

static EMPTY_NODE_STR: &'static str = "<empty>";

impl<T> Storage<T> where T: Resource {
    pub fn new(capacity: u32) -> Self {
        assert!(capacity > 0);
        let mut nodes = Vec::<ItemNode<T>>::with_capacity(capacity as usize);
        for i in 0..capacity {
            nodes.push(ItemNode {
                item: None,
                next_index: (i + 1) as u32,
                generation: 0,
                name: String::from(EMPTY_NODE_STR)
            });
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
        self.nodes.reserve_exact(capacity as usize);
        for i in capacity..2*capacity {
            self.nodes.push(ItemNode {
                item: None,
                next_index: i + 1,
                generation: 0,
                name: String::from(EMPTY_NODE_STR)
            });
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn capacity(&self) -> u32 {
        self.nodes.capacity() as u32
    }

    pub fn insert(&mut self, name: &str, item: T) -> ResourceID<T> {
        if self.first_available == self.capacity() {
            self.expand();
        }
        let new_index = self.first_available;

        let new_generation = {
            let node = &mut self.nodes[new_index as usize];
            self.first_available = node.next_index;

            assert!(node.item.is_none());

            node.item = Some(item);
            node.generation += 1;
            node.name = name.to_string();

            node.generation
        };

        self.name_mappings.insert(name.to_string(), new_index);

        let node_ref = ResourceID {
            index: new_index,
            generation: new_generation,
            tid: T::tid(),
            phantom: PhantomData
        };

        self.size += 1;

        node_ref
    }

    pub fn has(&self, item_ref: ResourceID<T>) -> bool {
        let node = &self.nodes[item_ref.index as usize];
        node.item.is_some() && node.generation == item_ref.generation
    }
    
    pub fn get(&self, item_ref: ResourceID<T>) -> &T {
        let node = &self.nodes[item_ref.index as usize];
        assert!(node.item.is_some());
        assert_eq!(node.generation, item_ref.generation);
        node.item.as_ref().unwrap()
    }

    pub fn get_mut(&mut self, item_ref: ResourceID<T>) -> &mut T {
        let node = &mut self.nodes[item_ref.index as usize];
        assert!(node.item.is_some());
        assert_eq!(node.generation, item_ref.generation);
        node.item.as_mut().unwrap()
    }

    pub fn get_by_name(&self, name: &str) -> Option<(&T, ResourceID<T>)> {
        self.name_mappings.get(name).map(|index| {
            let node = &self.nodes[*index as usize];
            assert!(node.item.is_some());
            (node.item.as_ref().unwrap(), ResourceID {
                index: *index,
                generation: node.generation,
                tid: T::tid(),
                phantom: PhantomData
            })
        })
    }

    pub fn get_mut_by_name(&mut self, name: &str) -> Option<(&mut T, ResourceID<T>)> {
        let index = match self.name_mappings.get(name) {
            Some(index) => *index,
            None => { return None; }
        };
        let node = &mut self.nodes[index as usize];
        assert!(node.item.is_some());
        Some((node.item.as_mut().unwrap(), ResourceID {
            index,
            generation: node.generation,
            tid: T::tid(),
            phantom: PhantomData
        }))
    }

    pub fn release(&mut self, item_ref: ResourceID<T>) {
        let node = &mut self.nodes[item_ref.index as usize];
        assert!(node.item.is_some());
        node.item = None;
        node.next_index = self.first_available;

        self.first_available = item_ref.index;
        self.size -= 1;
        self.name_mappings.remove(&node.name);
    }

    pub fn release_by_name(&mut self, name: &str) {
        // Trying to work around lexical lifetimes :(
        let index = match self.name_mappings.get(name) {
            Some(index) => { *index }
            None => { return; } // TODO: report error instead of failing silently...
        };

        let node = &mut self.nodes[index as usize];
        assert!(node.item.is_some());
        node.item = None;
        node.next_index = self.first_available;

        self.first_available = index;
        self.size -= 1;
        self.name_mappings.remove(name);
    }

    pub fn iterate<F>(&self, fun: F) where F : Fn(&T) -> () {
        for i in 0..self.capacity() {
            let node = &self.nodes[i as usize];
            if node.item.is_some() {
                fun(node.item.as_ref().unwrap())
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
    impl Resource for TestData1 {
        fn tid() -> u16 { 1 }
    }

    #[derive(Debug)]
    struct TestData2(i32);
    impl Resource for TestData2 {
        fn tid() -> u16 { 2 }
    }

    #[test]
    fn test_storage_new() {
        let mut storage = Storage::<TestData1>::new(8);
        assert_eq!(storage.capacity(), 8);
    }

    #[test]
    fn test_storage_insert_get() {
        let mut storage = Storage::new(8);
        
        let alice_ref = storage.insert("alice", (1, 3.0, "Alice".to_string()));

        assert_eq!(*storage.get(alice_ref), (1, 3.0, "Alice".to_string()));
        assert_eq!(*storage.get_by_name("alice").unwrap().0, (1, 3.0, "Alice".to_string()));

        assert!(storage.get_by_name("bob").is_none());
    }

    #[test]
    #[should_panic]
    fn test_storage_get_panic() {
        let mut storage = Storage::new(8);

        let alice_ref = storage.insert("alice", (1, 3.0, "Alice".to_string()));

        let invalid_ref = ResourceID::<TestData1> {
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
        
        let alice_ref = storage.insert("alice", (1, 3.0, "Alice".to_string()));

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

        let alice_ref = storage.insert("alice", (1, 3.0, "Alice".to_string()));
        let bob_ref = storage.insert("bob", (2, 4.0, "Bob".to_string()));

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
            // println!("Releasing item {}", i);
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
