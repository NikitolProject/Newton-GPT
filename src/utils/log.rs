use std::sync::{Arc, Mutex};

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use chrono::prelude::*;

pub async fn log_to_file(message: &str, messages: &Arc<Mutex<Vec<String>>>) -> std::io::Result<()> {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%dT%H:%M:%S").to_string();

    let formatted_message = format!("{} - {}", formatted_time, message);

    {
        let mut messages_guard = messages.lock().unwrap();
        messages_guard.push(formatted_message.to_owned());
    }

    let path = "bot.log";

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .await?;
        
    let mut file = tokio::io::BufWriter::new(file);

    file.write_all(formatted_message.as_bytes()).await?;
    file.write_all(b"\n").await?;
    file.flush().await
}

