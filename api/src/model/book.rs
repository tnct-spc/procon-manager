use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

impl From<CreateBookRequest> for kernel::model::book::event::CreateBook {
    fn from(request: CreateBookRequest) -> Self {
        Self {
            title: request.title,
            author: request.author,
            isbn: request.isbn,
            description: request.description,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub book_id: Uuid,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

impl From<kernel::model::book::Book> for BookResponse {
    fn from(book: kernel::model::book::Book) -> Self {
        Self {
            book_id: book.id,
            title: book.title,
            author: book.author,
            isbn: book.isbn,
            description: book.description,
        }
    }
}
