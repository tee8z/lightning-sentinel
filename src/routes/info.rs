use actix_web::{HttpResponse, Responder};

pub async fn info() -> impl Responder {
    HttpResponse::Ok().finish()
}
