use utoipa::OpenApi;

use crate::handler::{
    auth::ApiDoc as AuthApiDoc, checkout::ApiDoc as CheckoutApiDoc, health::ApiDoc as HealthApiDoc,
    item::ApiDoc as ItemApiDoc, user::ApiDoc as UserApiDoc,
};

pub fn build_openapi() -> utoipa::openapi::OpenApi {
    let mut api_doc = HealthApiDoc::openapi();
    api_doc.merge(AuthApiDoc::openapi());
    api_doc.merge(CheckoutApiDoc::openapi());
    api_doc.merge(ItemApiDoc::openapi());
    api_doc.merge(UserApiDoc::openapi());
    api_doc
}
