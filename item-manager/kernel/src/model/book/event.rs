use crate::model::id::BookId;

pub struct CreateBook {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

#[derive(Debug)]
pub struct UpdateBook {
    pub book_id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

#[derive(Debug)]
pub struct DeleteBook {
    pub book_id: BookId,
}
