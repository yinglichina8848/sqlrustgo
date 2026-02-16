//! Page management for storage engine

/// Page structure
#[derive(Debug, Clone)]
pub struct Page {
    pub page_id: u32,
    pub data: Vec<u8>,
}

impl Page {
    /// Create a new page
    pub fn new(page_id: u32) -> Self {
        Self {
            page_id,
            data: vec![0u8; 4096],
        }
    }

    /// Get page ID
    pub fn page_id(&self) -> u32 {
        self.page_id
    }

    /// Get page size
    pub fn size() -> usize {
        4096
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new(1);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.data.len(), 4096);
    }

    #[test]
    fn test_page_size() {
        assert_eq!(Page::size(), 4096);
    }

    #[test]
    fn test_page_new_zero_id() {
        let page = Page::new(0);
        assert_eq!(page.page_id(), 0);
    }

    #[test]
    fn test_page_data_initialized_to_zero() {
        let page = Page::new(1);
        // Check first and last bytes are zero
        assert_eq!(page.data[0], 0);
        assert_eq!(page.data[4095], 0);
    }

    #[test]
    fn test_page_debug() {
        let page = Page::new(42);
        let debug_str = format!("{:?}", page);
        assert!(debug_str.contains("42"));
    }
}
