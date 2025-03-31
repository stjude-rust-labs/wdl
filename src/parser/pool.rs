use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A memory pool for AST nodes to reduce allocation overhead
pub struct NodePool<T> {
    pool: RefCell<HashMap<usize, Vec<T>>>,
}

impl<T> NodePool<T> {
    pub fn new() -> Self {
        NodePool {
            pool: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, size: usize) -> Option<T> {
        let mut pool = self.pool.borrow_mut();
        let bucket = pool.get_mut(&size)?;
        bucket.pop()
    }

    pub fn put(&self, value: T, size: usize) {
        let mut pool = self.pool.borrow_mut();
        let bucket = pool.entry(size).or_insert_with(Vec::new);
        bucket.push(value);
    }
}

pub type SharedNodePool<T> = Rc<NodePool<T>>;

pub fn create_shared_pool<T>() -> SharedNodePool<T> {
    Rc::new(NodePool::new())
}