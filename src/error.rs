use rocket::http::ContentType;
use rocket::response::{self, Responder, Response};
use rocket::{http::Status, Request};
use serde::Serialize;
use std::fmt::Debug;
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("i/o {0}")]
    Io(#[from] std::io::Error),

    #[error("path denied {0}")]
    PathDenied(String),

    #[error("failed to decode path {}: {0}")]
    UrlDecode(String)
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Res<T>(pub T);

impl<T> From<T> for Res<T> {
    fn from(t: T) -> Self {
        Res(t)
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        #[derive(Serialize)]
        struct ResponseInner {
            err: String,
        }

        let inner = ResponseInner {
            err: self.to_string(),
        };

        match serde_json::to_string(&inner) {
            Err(err) => {
                error!(
                    "Failed to encode json response for error: {} error: {}",
                    inner.err, err
                );
                Err(Status::InternalServerError)
            }
            Ok(json) => Response::build()
                .sized_body(json.len(), Cursor::new(json))
                .status(Status::InternalServerError)
                .header(ContentType::JSON)
                .ok(),
        }
    }
}

impl<'r, T> Responder<'r, 'static> for Res<T>
where
    T: Serialize + Debug,
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        #[derive(Serialize)]
        struct ResponseInner<T: Serialize> {
            ok: T,
        }

        let inner = ResponseInner { ok: self.0 };

        match serde_json::to_string(&inner) {
            Err(err) => {
                error!(
                    "Failed to encode json response for: {:?}, error: {}",
                    inner.ok, err
                );
                Err(Status::InternalServerError)
            }
            Ok(json) => Response::build()
                .sized_body(json.len(), Cursor::new(json))
                .status(Status::InternalServerError)
                .header(ContentType::JSON)
                .ok(),
        }
    }
}

/*
impl<'r, T: MyStruct> Responder<'r, 'static> for T
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
*/
