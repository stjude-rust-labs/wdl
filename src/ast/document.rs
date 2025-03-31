// ... existing code ...

use std::cell::RefCell;

pub struct Document {
    // ... existing fields ...
    lazy_imports: RefCell<Option<Vec<Import>>>,
    lazy_structs: RefCell<Option<Vec<Struct>>>,
    // ... other sections that can be lazily parsed ...
}

impl Document {
    // ... existing methods ...
    
    pub fn imports(&self) -> &[Import] {
        if self.lazy_imports.borrow().is_none() {
            // Parse imports on demand
            let imports = self.parse_imports();
            *self.lazy_imports.borrow_mut() = Some(imports);
        }
        
        self.lazy_imports.borrow().as_ref().unwrap()
    }
    
    // Similar methods for other sections
    // ...
}

// ... existing code ...