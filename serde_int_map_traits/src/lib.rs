mod impls;

pub trait UnknownKeyHandler {
    type ValueType;

    fn new() -> Self;
    fn num_items(&self) -> usize;
    fn iter(&self) -> std::collections::hash_map::Iter<i64, Self::ValueType>;
    fn handles_key(&self, key: i64) -> bool;
    fn fill_value(&mut self, key: i64, value: Self::ValueType);
}
