pub trait BitArray {
    fn set_true(&mut self, index: usize);
    fn set_false(&mut self, index: usize);
    fn get(&self, index: usize) -> bool;
}

pub struct BitArray64 {
    value: u64,
}

impl BitArray64 {
    pub fn new() -> Self {
        BitArray64 { value: 0 }
    }
    pub fn iter(&self) -> BitArray64Iter {
        BitArray64Iter { count: 0, value: &self }
    }
}

impl BitArray for BitArray64 {
    fn set_true(&mut self, index: usize) {
        assert!(index >= 0 && index < 64);
        self.value |= (1 << index);
    }
    fn set_false(&mut self, index: usize) {
        assert!(index > 0 && index < 64);
        self.value &= !(1 << index);
    }
    fn get(&self, index: usize) -> bool {
        assert!(index >= 0 && index < 64);
        self.value & (1 << index) != 0
    }
}

pub struct BitArray64Iter<'a> {
    count: usize,
    value: &'a BitArray64
}

impl<'a> Iterator for BitArray64Iter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let count = self.count;
        self.count += 1;
        if self.value.get(count) {
            Some(count)
        }
        else {
            None
        }
    }
}

