use rocket::http::ContentType;
use rocket::response::{self, Responder, Response};
use rocket::{http::Status, Request};
use serde::Serialize;
use std::fmt::Debug;
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("I/O {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ApiResponse<T: Serialize + Debug>(pub Result<T>);

impl<'r, T> Responder<'r, 'static> for ApiResponse<T>
where
    T: Serialize + Debug,
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        #[derive(Serialize)]
        struct ResponseInner<T: Serialize> {
            ok: Option<T>,
            err: Option<String>,
        }

        let inner = self.0.map_or_else(
            |e| ResponseInner {
                ok: None,
                err: Some(e.to_string()),
            },
            |t| ResponseInner {
                ok: Some(t),
                err: None,
            },
        );

        let (body, status) = match serde_json::to_string(&inner) {
            Err(err) => {
                error!(
                    "Failed to encode json response for: {:?}, error: {}",
                    self.0, err
                );
                return Err(Status::InternalServerError);
            }
            Ok(json) => (json, Status::Ok),
        };

        Response::build()
            .sized_body(body.len(), Cursor::new(body.clone()))
            .status(status)
            .header(ContentType::JSON)
            .ok()
    }
}
