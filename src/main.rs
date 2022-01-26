#[macro_use]
extern crate rocket;

pub mod error;

use std::path::PathBuf;

use error::*;
use log::warn;
use rocket::{http::uri::Uri, State};

struct Config {
    mount: PathBuf,
}

#[get("/")]
fn hello() -> &'static str {
    "hello, world!"
}

#[get("/available/<unsanitized_req_path>")]
async fn available(
    unsanitized_req_path: &str,
    config: &State<Config>,
) -> Result<Res<Vec<(String, &'static str)>>> {
    //TODO: Use memory cache of files to get listing faster
    let unsanitized_uri =
        Uri::parse_any(unsanitized_req_path).map_err(|e| Error::UrlDecode(e.to_string()))?;
    info!("{:?}", unsanitized_uri);

    // We need to make the absolute path the user gives a relative path so we can push it to the
    // mount point
    let mut tmp = String::from(".");
    tmp.push_str(unsanitized_uri.to_string().as_str());

    let unsanitized_path = PathBuf::from(tmp);
    let unsanitized_path = config.mount.join(unsanitized_path);
    //We need to canonicalize the path _before_ we check if it is within our mount point so that
    //`../` and other nasty exploits are resolved.
    let unsanitized_path = std::fs::canonicalize(unsanitized_path)?;
    if !unsanitized_path
        .as_path()
        .starts_with(config.mount.as_path())
    {
        warn!("User requesting dirty path: {}", unsanitized_uri);
        warn!("  Mount at: {}", config.mount.to_string_lossy());
        warn!(
            "  Would have read path: {}",
            unsanitized_path.to_string_lossy()
        );
        return Err(Error::PathDenied(unsanitized_req_path.into()));
    }
    // The sanitized path starts with the mount point so it is safe to access
    let sanitized_path = /* unsafe { */ unsanitized_path /* } */;
    info!("Using sanitized path: {}", sanitized_path.to_string_lossy());

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
                if let Some(s) = entry.file_name().to_str() {
                    result.push((s.to_owned(), kind));
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
            eprintln!("Environment variable `MOUNT_POINT` not set!");
            error!("Environment variable `MOUNT_POINT` not set! Please set before running");
            std::process::exit(1);
        }
    };
    let mount = std::fs::canonicalize(mount).expect("Failed to find absolute path for mount point");
    info!("Using path: {}", mount.to_string_lossy());
    let config = Config { mount };
    rocket::build()
        .mount("/", routes![hello])
        .mount("/api/v1", routes![available])
        .manage(config)
}
