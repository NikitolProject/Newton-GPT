use std::sync::{Arc, Mutex};

use serenity::builder::CreateApplicationCommand;
use serenity::json::{JsonMap, json};
use serenity::model::channel::Message;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
// use serenity::model::prelude::interaction::application_command::CommandDataOption;

async fn create_new_thread(_ctx: &Context, _command: &ApplicationCommandInteraction, _message: Message, _title: String) {
    let mut options = JsonMap::new();

    options.insert("name".to_string(), json!(_title));

    let _ = match _ctx.http.create_public_thread(
        _message.channel_id.as_u64().to_owned(),
        _message.id.as_u64().to_owned(),
        &options
    ).await {
        Ok(v) => v,
        Err(e) => {
            println!("Cannot create new thread: {:#?}", e);
            return
        }
    };
}

pub async fn run(_ctx: &Context, _messages: &Arc<Mutex<Vec<String>>>, _command: &ApplicationCommandInteraction) -> String {
    // println!("{:#?}", _command);
    {
        let mut messages_guard = _messages.lock().unwrap();
        messages_guard.push(format!("[INFO] - Create new thread: {:#?}", _command).to_string());
    }

    let title = match &_command                            .data
        .options
        .first()
        .unwrap()
        .value {
            Some(v) => v.to_string(),
            _ => "Untitled".to_string()
        };

    let message = match _command
        .channel_id
        .send_message(&_ctx.http, |message| {
            message
                .content(
                    format!(
                        "Создаю новую беседу с названием: {}",
                        title
                    )
                )
        })
        .await {
            Ok(v) => v,
            Err(e) => {
                println!("Cannot send message: {:#?}", e);
                return "There was a server-side error. Please try again later.".to_string()
            }
        };

    create_new_thread(_ctx, _command, message, title).await;

    "Created!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("create_chat")
        .description("This command creates a separate thread for chatting with ChatGPT")
        .create_option(|option| {
            option
                .name("title")
                .description("Name of the new chat room")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

