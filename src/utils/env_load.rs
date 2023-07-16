use std::env;

use dotenv::dotenv;

static ENV_OPTIONS: &[&str; 6] = &[
    "DISCORD_TOKEN", "BOT_ID", "GUILD_ID", 
    "API_BASE", "API_KEY", "LOG_PATH"
];

pub async fn env_load() -> bool {
    let res = dotenv().ok();

    if !res.is_some() {
        panic!(
            "Create and populate an .env file in the root folder of the project where you run this executable, with all the required fields."
        );
    };

    for &option in ENV_OPTIONS {
        if env::var(option).is_err() {
            panic!("Fill in all missing fields in the env file!");
        } 
    }

    true
}

