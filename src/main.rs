#[macro_use]
extern crate rocket;

pub mod error;

use error::*;
use log::*;

#[get("/")]
fn hello() -> &'static str {
    "hello, world!"
}

#[get("/available/{path}")]
async fn available(path: String) -> Result<Vec<(String, &'static str)>> {
    //TODO: Use memory cache of files to get listing faster
    let mut it = tokio::fs::read_dir(path).await?;
    let mut result = Vec::new();
    while let Some(entry) = it.next_entry().await? {
        //Hide entries that we can't read
        match entry.metadata().await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    result.push((entry.path()));
                }
            }
            Err(err) => {
                warn!("Failed to read metadata of file {}", err);
            }
        }
    }
    Ok(result)
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
        .mount("/", routes![hello])
        .mount("/api/v1", routes![available])
}
