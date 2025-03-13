use crate::model::id::ItemId;

pub struct CreateBook {
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

#[derive(Debug)]
pub struct UpdateBook {
    pub item_id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

#[derive(Debug)]
pub struct DeleteBook {
    pub item_id: ItemId,
}
