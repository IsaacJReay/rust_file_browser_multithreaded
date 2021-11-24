#[derive(Debug)]
pub struct ItemMetaData {
    pub item_last_modify_date: String,
    pub item_type: String,
    pub item_size: usize,
}

impl ItemMetaData {
    pub fn get_type(self) -> String {
        self.item_type
    }
}

#[derive(Debug)]
pub struct DirectoryInfo {
    pub item_path: String,
    pub item_metadata: ItemMetaData,
    pub sub_path_items: Vec<Box<DirectoryInfo>>,
}