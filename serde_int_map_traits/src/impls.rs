use super::UnknownKeyHandler;

use std::collections::hash_map::Iter;
use std::collections::HashMap;

impl<VT> UnknownKeyHandler for HashMap<u32, VT> {
    type ValueType = VT;

    fn new() -> Self {
        HashMap::new()
    }

    fn num_items(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Iter<u32, Self::ValueType> {
        self.iter()
    }

    fn handles_key(&self, _key: u32) -> bool {
        true
    }

    fn fill_value(&mut self, key: u32, value: Self::ValueType) {
        self.insert(key, value);
    }
}
