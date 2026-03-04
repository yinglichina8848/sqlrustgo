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
    fn test_page_data_access() {
        let mut page = Page::new(1);
        // Write some data
        page.data[0] = 0xAB;
        page.data[1] = 0xCD;
        // Read it back
        assert_eq!(page.data[0], 0xAB);
        assert_eq!(page.data[1], 0xCD);
    }

    #[test]
    fn test_page_default_size() {
        assert_eq!(Page::size(), 4096);
    }
}
