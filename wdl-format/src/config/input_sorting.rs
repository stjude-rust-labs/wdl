//! Configuration for input sorting

/// Whether to sort the inputs or not
#[derive(Clone, Copy, Debug)]
pub enum SortInputs {
    /// Sort the inputs
    Sort,
    /// Do not sort the inputs
    NoSort,
}

impl Default for SortInputs {
    fn default() -> Self {
        SortInputs::Sort
    }
}

impl SortInputs {
    pub fn try_new(sort_inputs :bool) -> Self {
        if sort_inputs {
            SortInputs::Sort
        } else {
            SortInputs::NoSort
        }
    }
    
    pub fn get(&self) -> bool{
        match self {
            SortInputs::Sort => true,
            SortInputs::NoSort => false,
        }
    }
}
