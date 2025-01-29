use dioxus::prelude::*;
#[cfg(feature = "server")]
use futures::TryStreamExt;
#[cfg(feature = "server")]
use mongodb::{options::ClientOptions, Client, bson::oid::ObjectId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct WordSet {
    pub no_sentence: String,
    pub tr_sentence: String,
    pub no_word: String,        // Base form (infinitive)
    pub tr_word: String,
    pub conjugated_word: String, // Form used in the sentence
    #[serde(default)]  // This will default to 0 if the field is missing
    pub correct_attempts: i32,
}

#[server(GetWordsSet)]
pub async fn get_words_set() -> Result<Vec<WordSet>, ServerFnError> {
    println!("Fetching word context..."); // Debug log
    dotenv::dotenv().ok();
    let client_uri =
        env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    println!("Using MongoDB URI: {}", client_uri); // Debug connection string

    let client_options = ClientOptions::parse(client_uri)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse MongoDB URI: {e}")))?;
    let client = Client::with_options(client_options)
        .map_err(|e| ServerFnError::new(format!("Failed to create MongoDB client: {e}")))?;
    
    // List databases to verify connection
    println!("Connected to MongoDB. Available databases:");
    if let Ok(dbs) = client.list_database_names().await {
        for db in dbs {
            println!("- {}", db);
        }
    }

    let db = client.database("husker");
    let collection = db.collection::<WordSet>("no_tr");

    // List collections in the database
    println!("Collections in 'husker' database:");
    if let Ok(collections) = db.list_collection_names().await {
        for coll in collections {
            println!("- {}", coll);
        }
    }

    let filter = mongodb::bson::doc! {};
    
    // First try to get raw documents
    let mut raw_cursor = db.collection::<mongodb::bson::Document>("no_tr")
        .find(filter.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to execute raw find: {e}")))?;

    // Print the first raw document to see its structure
    if let Ok(Some(first_doc)) = raw_cursor.try_next().await {
        println!("First document structure: {:#?}", first_doc);
    }

    // Now try the actual query
    let mut cursor = collection
        .find(filter)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to execute find: {e}")))?;

    let mut words = Vec::new();
    while let Some(word_result) = cursor
        .try_next()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get next document: {e}")))?
    {
        words.push(word_result);
    }

    println!("Found {} words", words.len()); // Debug log
    Ok(words)
}
