use std::{fs, error::Error};

use tokio::fs as tokio_fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

static DATASTORAGE_FOLDER_NAME: &str = "data";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: u64,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Users {
    pub users: Vec<User>,
}

impl Users {
    pub async fn default() -> Result<Users, Box<dyn Error>> {
        let bson_bytes = tokio_fs::read(
            &format!("{}/users.bson", DATASTORAGE_FOLDER_NAME)
        ).await?;
        let document = bson::from_slice(&bson_bytes)?;
        let users: Users = bson::from_bson(Bson::Document(document))?;
        Ok(users)
    }

    pub async fn write_users_datastorage(&self) -> Result<(), Box<dyn Error>> {
        let document = bson::to_document(&self)?;
        let bson_bytes = bson::to_vec(&document)?;
        
        // tokio_fs::write(
        //     &format!("{}/users.bson", DATASTORAGE_FOLDER_NAME), bson_bytes
        // ).await?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&format!("{}/users.bson", DATASTORAGE_FOLDER_NAME))
            .await?;
        
        let mut file = tokio::io::BufWriter::new(file);

        file.write_all(&bson_bytes).await?;
        file.flush().await.unwrap();
        
        Ok(())
    }

    pub fn find_user_by_id(&self, user_id: u64) -> Option<&User> {
        self.users.iter().find(|user| user.user_id == user_id)
    }

    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn update_user(&mut self, user_id: u64, new_model: String) -> bool {
        if let Some(user) = self.users.iter_mut().find(|user| user.user_id == user_id) {
            user.model = new_model;
            true
        } else {
            false
        }
    }

    pub fn delete_user(&mut self, user_id: u64) -> bool {
        if let Some(index) = self.users.iter().position(|user| user.user_id == user_id) {
            self.users.remove(index);
            true
        } else {
            false
        }
    }
}

pub async fn check_datastorage_exists() {
    if let Err(err) = fs::metadata(DATASTORAGE_FOLDER_NAME) {
        if err.kind() == std::io::ErrorKind::NotFound {
            match fs::create_dir(DATASTORAGE_FOLDER_NAME) {
                Ok(()) => {},
                Err(e) => panic!("Failed to create a folder to store data: {}", e),
            }
        }
    }

    check_data_users_file()
        .await.expect("Field check users.bson in datastorage");
}

async fn check_data_users_file() -> Result<(), Box<dyn Error>> {
    let file_path = &format!("{}/users.bson", DATASTORAGE_FOLDER_NAME);

    if tokio_fs::metadata(file_path).await.is_err() {
        let document = doc! {
            "users": []
        };

        let bson_bytes = bson::to_vec(&document)?;
        tokio_fs::write(file_path, bson_bytes).await?;
    }

    Ok(())
}

