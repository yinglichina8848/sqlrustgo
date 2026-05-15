pub struct HashJoinSpillBuilder;

impl HashJoinSpillBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HashJoinSpillBuilder {
    fn default() -> Self {
        Self::new()
    }
}
