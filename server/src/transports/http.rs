use actix_web::{post, web::Data, HttpResponse, Responder};
use roblib::cmd::Concrete;
use roblib_parsing::{Readable, Writable, SEPARATOR};

use crate::{cmd::execute_concrete, AppState};

#[post("/cmd")]
pub(crate) async fn index(body: String, state: Data<AppState>) -> impl Responder {
    let concrete = match Concrete::parse_text(&mut body.split(SEPARATOR)) {
        Ok(c) => c,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    match execute_concrete(concrete, state.robot.clone()).await {
        Ok(Some(s)) => {
            let mut b = String::new();

            match s.write_text(&mut b) {
                Ok(()) => {
                    b.pop();
                    HttpResponse::Ok().body(b)
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Ok(None) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
