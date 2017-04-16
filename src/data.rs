#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub id: i32,
    pub name: String,
    pub read: bool,
    pub page: u32,
    pub genre: String,
    pub author: String,
}
