use std::sync::Arc;

use actix_web::{post, web::Data, HttpResponse, Responder};
use roblib::{cmd::Concrete, text_format};

use crate::{cmd::execute_concrete, Robot};

#[post("/cmd")]
pub(crate) async fn index(body: String, robot: Data<Arc<Robot>>) -> impl Responder {
    let concrete: Concrete = match text_format::de::from_str(&body) {
        Ok(c) => c,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let mut ret = String::new();
    if let Err(e) = execute_concrete(
        concrete,
        robot.as_ref().clone(),
        &mut text_format::ser::Serializer::new(&mut ret),
    )
    .await
    {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok().body(ret)
}
