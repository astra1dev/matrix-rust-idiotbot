mod matrix;

use crate::matrix::matrix::login_and_sync;
use matrix_sdk::ruma::OwnedEventId;
use std::{collections::HashMap, env, process::exit};

type Username = String;
type EventId = OwnedEventId;
type Reason = String;
type Upvotes = i32;
type IdiotExposer = String;

struct Idiots {
    hash_map: HashMap<Username, HashMap<EventId, (Reason, Upvotes, IdiotExposer)>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // parse the command line for homeserver, username and password
    let (Some(homeserver_url), Some(username), Some(password)) =
        (env::args().nth(1), env::args().nth(2), env::args().nth(3))
    else {
        eprintln!(
            "Usage: {} <homeserver_url> <username> <password>",
            env::args().next().unwrap()
        );
        exit(1)
    };

    login_and_sync(homeserver_url, &username, &password).await?;
    Ok(())
}
