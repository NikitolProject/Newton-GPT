use std::env;

use chatgpt::{prelude::*, types::Role};

use serenity::model::channel::Message;

pub async fn send_gpt_message(model: &str, history: Vec<ChatMessage>) -> String {
    let api_base = env::var("API_BASE") 
        .expect("API BASE must be not empty in enviroment!");

    let api_key = env::var("API_KEY")
        .expect("API KEY must be not empty in enviroment!");

    let client = match ChatGPT::new_with_config(
        api_key,
        ModelConfigurationBuilder::default()
            .api_url(Url::parse(&api_base).unwrap())
            .temperature(1.0)
            .engine(if model == "\"gpt-3.5-turbo\"" { ChatGPTEngine::Gpt35Turbo } else { ChatGPTEngine::Gpt4 })
            .build()
            .unwrap(),
    ) {
        Ok(v) => v,
        Err(e) => {
            println!("{:#?}", e);
            return "Error.".to_string()
        }
    };

    let mut conversation = client.new_conversation();

    for message in history.iter() {
        conversation.history.push(message.to_owned());
    }

    // println!("{:#?}", history);
    let response = match conversation
        .send_message(history.first().unwrap().content.to_string())
        .await {
            Ok(v) => v,
            Err(e) => {
                println!("{:#?}", e);
                return "Error.".to_string()
            }
        };

    response.message().content.to_string()
}

pub fn get_gpt_history_from_messages(_history: Vec<Message>) -> Vec<ChatMessage> {
    let mut history = vec![];

    let bot_id = env::var("BOT_ID")
        .unwrap_or("0".to_owned());

    for message in _history.iter() {
        if bot_id == message.author.id.to_string() {
            history.push(ChatMessage { role: Role::Assistant, content: message.content.to_string() });
            continue;
        }

        history.push(ChatMessage { role: Role::User, content: message.content.to_string() });
    };

    history
}

