use log::warn;
use rocket::{
    http::{uri::Uri, ContentType},
    response::{self, Responder},
    Request, Response,
};
use std::path::PathBuf;

use crate::{Config, Error, Result};

/// Canonicalizes path, and ensures that it is not a symlink. Returning a naw sanitized path.
///
/// This function will return Err(...) under the following circumstances:
/// - Points to a symlink, `Err(Error::PathDenied)` will be returned
/// - Does not exist or cannot be canonicalized, `Err(Error::Io(...))`
/// - Points outsize of the configured mount point after canonicalization,
///     `Err(Error::PathDenied)` will be returned
/// Otherwise the (now safe to access) absolute will be returned
pub fn sanitize_path(unsanitized_path: PathBuf, config: &Config) -> Result<PathBuf> {
    //Check for symlinks and always show file not found
    match std::fs::symlink_metadata(&unsanitized_path) {
        Ok(data) if data.is_symlink() => {
            // Return not found because we don't support reading symlinks
            return Err(Error::Io(std::io::Error::from_raw_os_error(2)));
        }
        _ => {
            //Continue to mount point check
        }
    }

    //We need to canonicalize the path _before_ we check if it is within our mount point so that
    //`../` and other nasty exploits can be detected.
    let unsanitized_path = std::fs::canonicalize(unsanitized_path)?;
    if !unsanitized_path
        .as_path()
        .starts_with(config.mount.as_path())
    {
        warn!(
            "Preventing read of dirty path: {}",
            unsanitized_path.to_string_lossy()
        );
        warn!("  Mount at: {}", config.mount.to_string_lossy());
        return Err(Error::PathDenied);
    }
    // The sanitized path starts with the mount point so it is safe to access
    let sanitized_path = /* unsafe { */ unsanitized_path /* } */;
    Ok(sanitized_path)
}

/// Sanitizes a Url encoded relative path using the mount point, returning the absolute to the response, if it
/// is allowed to be accessed.
///
/// This function will return Err(...) under the following circumstances:
/// - Points to a symlink, `Err(Error::PathDenied)` will be returned
/// - Does not exist or cannot be canonicalized, `Err(Error::Io(...))`
/// - Points outsize of the configured mount point after canonicalization,
///     `Err(Error::PathDenied)` will be returned
/// Otherwise the (now safe to access) absolute will be returned
pub fn sanitize_relative(unsanitized_req_path: &str, config: &Config) -> Result<PathBuf> {
    let unsanitized_uri =
        Uri::parse_any(unsanitized_req_path).map_err(|e| Error::UrlDecode(e.to_string()))?;

    // We need to make a path that looks like a relative path so that it can be joined with the mount
    let unsanitized_str = unsanitized_uri.to_string();
    let mut unsanitized_rel_path = String::new();
    if unsanitized_str.starts_with('/') {
        unsanitized_rel_path.push('.');
    } else if !unsanitized_str.starts_with("./") {
        unsanitized_rel_path.push_str("./");
    } else {
        //Nop. This should concatenate just fine
    }
    unsanitized_rel_path.push_str(&unsanitized_str);

    let unsanitized_path = PathBuf::from(unsanitized_rel_path);
    let unsanitized_path = config.mount.join(unsanitized_path);

    // `sanitize_path` handles the sanitization from here
    /* unsafe { */
    sanitize_path(unsanitized_path, config) /* } */
}

pub struct StreamedFile(pub tokio::fs::File);

impl<'r> Responder<'r, 'static> for StreamedFile {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        Response::build()
            .streamed_body(self.0)
            .header(ContentType::new("application", "octet-stream"))
            .ok()
    }
}

pub struct EasyResponse(pub Response<'static>);

impl<'r> Responder<'r, 'static> for EasyResponse {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        Ok(self.0)
    }
}
