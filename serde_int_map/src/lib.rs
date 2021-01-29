pub trait UnknownKeyHandler {
    type ValueType;

    fn new() -> Self;
    fn num_items(&self) -> usize;
    fn iter(&self) -> std::collections::hash_map::Iter<u32, Self::ValueType>;
    fn handles_key(&self, key: u32) -> bool;
    fn fill_value(&mut self, key: u32, value: Self::ValueType);
}
