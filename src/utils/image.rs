use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use reqwest::multipart;
use reqwest::header::{AUTHORIZATION, USER_AGENT};

use rand::Rng;
use crypto::digest::Digest;
use crypto::md5::Md5;

use crate::utils::log::log_to_file;

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String
}

fn md5(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.input_str(input);
    let result = hasher.result_str();
    result.chars().rev().collect::<String>()
}

fn get_api_key(user_agent: &str) -> String {
    let mut rng = rand::thread_rng();
    let part1: u64 = rng.gen_range(0..10u64.pow(11));

    let part2 = md5(&(user_agent.to_owned() 
                    + &md5(&(user_agent.to_owned() 
                           + &md5(&(user_agent.to_owned() 
                                  + &part1.to_string() 
                                  + "x"))))));
    
    format!("tryit-{}-{}", part1.to_string(), part2)
}

fn convert_to_hashmap(value: &Value) -> Option<HashMap<String, String>> {
    match value {
        Value::Object(map) => Some(map.iter().map(|(k, v)| (k.clone(), v.to_string())).collect()),
        _ => None,
    }
}

pub async fn image_submission_check(message: &str, log_messages: &Arc<Mutex<Vec<String>>>) -> Result<(bool, String), (bool, String)> {
    let client = reqwest::Client::new();

    let message = format!("Is there a request in this post to generate a new image? As an answer, write two words of your choice: YES, if there is such a request, and NO, if there is no request to generate an image in the message.\nIn case your answer is YES, write what size image the user wants in WxH format without any extra words (if the size is not specified by the user - write 1024x1024).\nHere is the message itself:\n{}", message);

    let message = Message {
        role: "user".to_owned(),
        content: message
    };

    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

    let chat_style = multipart::Part::text("chat");

    let message_serialized = match serde_json::to_string(&vec![message]) {
        Ok(v) => v,
        Err(e) => {
            return Err(
                (false, format!("Cannot deserialized this message: {}", e))
            )
        }
    };

    log_to_file("[INFO] - Checking...", log_messages)
            .await.unwrap();

    let chat_history = multipart::Part::text(message_serialized);

    log_to_file("[INFO] - Chat history created...", log_messages)
        .await.unwrap();

    let form = multipart::Form::new()
        .part("chat_style", chat_style)
        .part("chatHistory", chat_history);

    log_to_file("[INFO] - Form created...", log_messages)
        .await.unwrap();

    let api_key = &get_api_key(user_agent);

    log_to_file(&format!("[INFO] - Created API key: {}", api_key), log_messages)
        .await.unwrap();

    let res = match client
        .post("https://api.deepai.org/chat_response")
        .header(USER_AGENT, user_agent)
        .header("api-key", api_key)
        .multipart(form)
        .send()
        .await {
            Ok(v) => v,
            Err(e) => {
                log_to_file(&format!("[ERROR] - Error with sending request: {:#?}", e), log_messages)
                    .await.unwrap();
                return Ok((false, "not sending request".to_owned()));
            }
        };

    log_to_file("[INFO] - Request sended...", log_messages)
            .await.unwrap();

    if res.status().is_success() {
        let content: String = res.text().await.unwrap();

        if content.starts_with("YES") {
            let content = str::replace(&content, "YES, ", "");
            let content = str::replace(&content, ".", "!");
            
            return Ok((true, content.to_owned()));
        }
    }

    log_to_file("[INFO] - Check complited.", log_messages)
            .await.unwrap();
    
    Ok((false, "".to_owned()))
}

pub async fn get_images(text: &str, size: &str, count: &i32, log_messages: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    let api_base = env::var("API_BASE_IMAGE") 
        .expect("API BASE must be not empty in enviroment!");

    let api_key = env::var("API_KEY")
        .expect("API KEY must be not empty in enviroment!");

    let client = reqwest::Client::new();

    let res: Value = client
        .post(api_base)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .json(
            &json!({
                "prompt": text,
                "n": count,
                "size": size
            })
        )
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut urls = vec![];

    let empty_map = &serde_json::Map::new();

    if !res.as_object().unwrap_or(empty_map).contains_key("data") {
        return vec!["Простите, но у меня не получилось ничего нарисовать. Попробуйте позже(".to_owned()]
    }

    let data = &res["data"];

    let empty_array = vec![];

    log_to_file(&format!("[INFO] - Images: {}", data), log_messages)
            .await.unwrap();

    for value in data.as_array().unwrap_or(&empty_array) {
        urls.push(value["url"].as_str().unwrap().to_owned());
    };

    urls
}

