

pub struct FallbackManager {
    fallback_enabled: bool,
}

impl FallbackManager {
    pub fn new() -> Self {
        Self {
            fallback_enabled: true,
        }
    }

    pub fn should_fallback(&self) -> bool {
        self.fallback_enabled
    }

    pub fn disable_fallback(&mut self) {
        self.fallback_enabled = false;
    }
}

impl Default for FallbackManager {
    fn default() -> Self {
        Self::new()
    }
}
