use std::sync::{Arc, Mutex};

use crate::utils::datastorage::{Users, User};

use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;

pub async fn run(_ctx: &Context, _command: &ApplicationCommandInteraction, _messages: &Arc<Mutex<Vec<String>>>) -> String {
    let mut users = match Users::default().await {
        Ok(v) => v,
        Err(_) => {
            // log_to_file(
            //     &format!("[ERROR] - Cannot open users.bson: {}", e), _messages
            // ).await.unwrap();

            return "Error in datastorage.".to_owned()
        }
    };

    let new_model = match &_command.data.options
        .first()
        .unwrap()
        .value {
            Some(v) => v,
            _ => {
                // log_to_file(
                //     "[ERROR] - Cannot fetch the model name", _messages
                // ).await.unwrap();

                return "Error fetch the model name.".to_owned()
            }
        };

    let res = users.update_user(_command.user.id.as_u64().to_owned(), new_model.to_string());
    
    if !res {
        users.add_user(
            User {
                user_id: _command.user.id.as_u64().to_owned(),
                model: new_model.to_string()
            }  
        );
    };

    match users.write_users_datastorage().await {
        Ok(_) => {},
        Err(e) => {
            // log_to_file(
            //     &format!("[ERROR] - Cannot update model for user: {}", e), _messages
            // ).await.unwrap();
            //
            let error = &format!("[ERROR] - Cannot update model for user: {}", e);
            return error.to_owned()
        }
    };

    "The GPT model update for you has been successfully completed!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("model")
        .description("Select a GPT model for your requests")
        .create_option(|option| {
            option
                .name("name")
                .description("Select the model name from the given options")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice(
                    "ChatGPT 3.5-turbo", "gpt-3.5-turbo"
                )
                .add_string_choice(
                    "ChatGPT 4", "gpt-4"
                )
        })
}

