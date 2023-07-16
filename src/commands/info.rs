use std::sync::{Arc, Mutex};

use crate::utils::datastorage::{Users, User};

use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(_ctx: &Context, _command: &ApplicationCommandInteraction, _messages: &Arc<Mutex<Vec<String>>>) -> String {
    let users = match Users::default().await {
        Ok(v) => v,
        Err(_) => {
            // log_to_file(
            //     &format!("[ERROR] - Cannot open users.bson: {}", e), _messages
            // ).await.unwrap();

            return "Error in datastorage.".to_owned()
        }
    };

    let mut model = "gpt-3.5-turbo";

    let current_user = users.find_user_by_id(_command.user.id.as_u64().to_owned());

    if current_user.is_none() {
        let mut users = match Users::default().await {
            Ok(v) => v,
            Err(_) => {
                return "Error in datastorage.".to_owned()
            }
        };

        users.add_user(
            User {
                user_id: _command.user.id.as_u64().to_owned(),
                model: model.to_string()
            }  
        );

        match users.write_users_datastorage().await {
            Ok(_) => {},
            Err(e) => {
                let error = &format!("[ERROR] - Cannot update model for user: {}", e);
                return error.to_owned()
            }
        };
    } else {
        model = &current_user.unwrap().model;
    };

    format!("The currently selected GPT model: {}", model).to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("info")
        .description("Gives information about the currently selected GPT model")
}

