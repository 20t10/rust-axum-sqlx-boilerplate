#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub page: u32,
    pub per_page: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}
