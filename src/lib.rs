pub mod ui;
pub mod utils;
pub mod commands;

use std::{env, sync::{Arc, Mutex}};

use crate::utils::{log::log_to_file, datastorage::{Users, User}};

// use std::io::Write;
// use chrono::Local;
// use env_logger::Builder;
// use log::LevelFilter;

use serenity::async_trait;
// use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler {
    messages: Arc<Mutex<Vec<String>>>
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _new_message: Message) {
        let bot_id = env::var("BOT_ID")
            .unwrap_or("0".to_owned())
            .parse()
            .unwrap_or(0);

        if _new_message.author.id == bot_id {
            return
        };

        let mut members = match _new_message
            .channel_id
            .get_thread_members(
                &_ctx.http
            )
            .await {
                Ok(v) => v,
                Err(_) => return
            };

        members.sort_by(|a, b| {
            a.join_timestamp.cmp(&b.join_timestamp)
        });

        if members.len() == 0 || members.first().expect("Not members").user_id.unwrap() != bot_id {
            return
        };

        log_to_file(&format!("[INFO] - Thread members: {:#?}", members), &self.messages)
            .await.unwrap();

        // println!("{:#?}", _new_message);
        log_to_file(&format!("[INFO] - Get new message from thread: {:#?}", _new_message), &self.messages)
            .await.unwrap();

        // if _new_message.mentions.iter().any(|m| m.id == bot_id) 
        //   || (_new_message.referenced_message.is_some() && _new_message.referenced_message.unwrap().author.id == bot_id) {
        let history = match _new_message.channel_id.messages(
            &_ctx.http, |message| {
                message.limit(30)
            }
        ).await {
            Ok(v) => v,
            Err(e) => {
                // println!("{:#?}", e);
                log_to_file(&format!("[WARN] - Can`t seen messages: {:#?}", e), &self.messages)
                    .await.unwrap();
                return
            }
        };
        let users = Users::default().await.unwrap();
        let mut model = "gpt-3.5-turbo";

        let current_user = users.find_user_by_id(_new_message.author.id.as_u64().to_owned());

        let mut users = Users::default().await.unwrap();

        if current_user.is_none() {
            let new_user = User {
                user_id: _new_message.author.id.as_u64().to_owned(),
                model: "gpt-3.5-turbo".to_string()
            };
            users.add_user(new_user);
            users.write_users_datastorage().await.unwrap();
        } else {
            model = &current_user.unwrap().model;
        }

        let copied_http_client = Arc::new(&_ctx.http);

        let typing = copied_http_client
            .start_typing(_new_message.channel_id.as_u64().to_owned())
            .expect("Error typing");

        let history = utils::gpt::get_gpt_history_from_messages(history);
        {
            let mut messages_guard = self.messages.lock().unwrap();
            messages_guard.push(format!("[INFO] - History: {:#?}", history));
        }
        let text = utils::gpt::send_gpt_message(model, history).await;
        
        typing.stop();

        match _new_message
            .channel_id
            .send_message(
                &_ctx.http, 
                |m| {
                    m.content(text.to_string())
                }
            )
            .await {
                Ok(v) => v,
                Err(e) => { 
                    // println!("Cannot send new message: {}", e);
                    log_to_file(&format!("[WARN] - Can`t send message: {:#?}", e), &self.messages)
                        .await.unwrap();
                    return
                }
            };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);
            log_to_file(&format!("[INFO] - Received command interaction: {:#?}", command), &self.messages)
                .await.unwrap();

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                "create_chat" => commands::create_chat::run(&ctx, &self.messages, &command).await,
                "model" => commands::model::run(&ctx, &command, &self.messages).await,
                "info" => commands::info::run(&ctx, &command, &self.messages).await,
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.ephemeral(true)
                                .content(content)
                        })
                })
                .await
            {
                // println!("Cannot respond to slash command: {}", why);
                log_to_file(&format!("[ERROR] - Cannot respond to slash command: {}", why), &self.messages)
                    .await.unwrap();
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        // println!("{} is connected!", ready.user.name);
        log_to_file(&format!("[INFO] - {} is connected!", ready.user.name), &self.messages)
            .await.unwrap();

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );
        
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::info::register(command))
                .create_application_command(|command| commands::model::register(command))
                .create_application_command(|command| commands::create_chat::register(command))
        })
        .await;

        // println!("I now have the following guild slash commands: {:#?}", commands);
        log_to_file(&format!("[INFO] - I now have the following guild slash commands: {:#?}", commands), &self.messages)
            .await.unwrap();
    }
}

pub async fn start_bot(messages: Arc<Mutex<Vec<String>>>) {
    // TODO: Add logging 

    // Builder::new()
    //     .format(|buf, record| {
    //         writeln!(buf,
    //             "{} [{}] - {}",
    //             Local::now().format("%Y-%m-%dT%H:%M:%S"),
    //             record.level(),
    //             record.args()
    //         )
    //     })
    //     .filter(None, LevelFilter::Info)
    //     .init();
    log_to_file("[INFO] - Starting bot...", &messages)
        .await.unwrap();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Build our client.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { messages })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        panic!("Client error: {:?}", why);
    }
}

