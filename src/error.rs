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

    #[error("not found")]
    PathDenied,

    #[error("failed to decode path {}: {0}")]
    UrlDecode(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Wrapper type for a successful result.
///
/// Implements [`rocket::response::Responder`] so that successful results can be easily serialized
/// as json
pub(crate) struct Res<T>(pub T);

impl<T> From<T> for Res<T> {
    fn from(t: T) -> Self {
        Res(t)
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(mut self, _: &'r Request<'_>) -> response::Result<'static> {
        if let Error::Io(io) = &self {
            if io.kind() == std::io::ErrorKind::NotFound {
                //Change all not found errors to path denied to obscure files that actually don't
                //exist, and files that we don't allow users to access
                self = Error::PathDenied;
            }
        }
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
