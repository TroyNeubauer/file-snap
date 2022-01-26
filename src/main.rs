#[macro_use]
extern crate rocket;

mod error;
mod util;

pub use error::*;

use std::path::PathBuf;

use log::warn;
use rocket::State;
use serde::Serialize;

pub struct Config {
    mount: PathBuf,
}

#[get("/")]
fn hello() -> &'static str {
    "hello, world!"
}

#[get("/read/<unsanitized_req_path>")]
async fn read<'r>(
    unsanitized_req_path: &str,
    config: &State<Config>,
) -> Result<util::StreamedFile> {
    let sanitized_path = util::sanitize_relative(unsanitized_req_path, config)?;

    let file = tokio::fs::File::open(sanitized_path).await?;
    Ok(util::StreamedFile(file))
}

#[derive(Serialize, Debug)]
struct DirListing {
    name: String,
    kind: &'static str,
}

#[get("/list")]
async fn list_no_param(config: &State<Config>) -> Result<Res<Vec<DirListing>>> {
    list("", config).await
}

#[get("/list/<unsanitized_req_path>")]
async fn list(unsanitized_req_path: &str, config: &State<Config>) -> Result<Res<Vec<DirListing>>> {
    let sanitized_path = util::sanitize_relative(unsanitized_req_path, config)?;

    //TODO: Use memory cache of files to get listing faster
    let mut it = tokio::fs::read_dir(sanitized_path).await?;
    let mut result = Vec::new();
    while let Some(entry) = it.next_entry().await? {
        //Hide entries that we can't read
        match entry.metadata().await {
            Ok(metadata) => {
                let kind = if metadata.is_dir() {
                    "d"
                } else if metadata.is_file() {
                    "f"
                } else {
                    //Symlink. Don't show
                    continue;
                };
                //Sanity check that we can access this path. Should never happen because we check
                //for symlinks above
                debug_assert!(util::sanitize_path(entry.path(), config).is_ok());
                if let Some(s) = entry.file_name().to_str() {
                    result.push(DirListing {
                        name: s.to_owned(),
                        kind,
                    });
                }
            }
            Err(err) => {
                warn!("Failed to read metadata of file {}", err);
            }
        }
    }
    Ok(Res(result))
}

#[launch]
fn rocket() -> _ {
    let _ = dotenv::dotenv();
    env_logger::init();

    let mount = match std::env::var_os("MOUNT_POINT") {
        Some(mount) => mount,
        None => {
            eprintln!("Environment variable `MOUNT_POINT` missing");
            error!("Environment variable `MOUNT_POINT` is not set! Please set before running");
            std::process::exit(1);
        }
    };
    let mount = std::fs::canonicalize(mount)
        .expect("Failed to find absolute path for mount point. Does it exist?");

    info!("Using mount point: {}", mount.to_string_lossy());
    let config = Config { mount };
    rocket::build()
        .mount("/", routes![hello])
        .mount("/api/v1", routes![list, list_no_param, read])
        .manage(config)
}

/*

// build.rs
use std::env;

pub fn main() {
    if Ok("release".to_owned()) == env::var("PROFILE") {
        panic!("I'm only panicking in release mode")
    }
}
*/
