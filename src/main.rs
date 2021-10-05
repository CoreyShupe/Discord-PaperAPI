extern crate paper_api;
extern crate tokio;

mod commands;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

use serenity::framework::standard::StandardFramework;
use serenity::Client;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    token: String,
}

fn load_config(path: &str) -> Config {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `AppInfo`.
    serde_json::from_reader(reader).expect("Config malformed.")
}

#[tokio::main]
async fn main() {
    let config = load_config("./etc/config.json");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .group(&commands::GENERAL_GROUP);

    let mut client = Client::builder(config.token)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
