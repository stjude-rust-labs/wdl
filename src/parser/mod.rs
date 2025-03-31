// ... existing code ...

use crate::parser::pool::{SharedNodePool, create_shared_pool};

pub struct Parser {
    // ... existing fields ...
    node_pool: SharedNodePool<Box<dyn AstNode>>,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        // ... existing code ...
        let node_pool = create_shared_pool();
        
        Parser {
            // ... existing fields ...
            node_pool,
        }
    }
    
    // Modify allocation methods to use the pool
    fn allocate_node<T: AstNode + 'static>(&self, node: T) -> Box<dyn AstNode> {
        let size = std::mem::size_of::<T>();
        if let Some(mut boxed) = self.node_pool.get(size) {
            // Reuse existing allocation
            unsafe {
                std::ptr::write(Box::into_raw(boxed) as *mut T, node);
                boxed
            }
        } else {
            // Create new allocation
            Box::new(node)
        }
    }
    
    // ... existing methods ...
}

// ... existing code ...