#![allow(dead_code)]

use actix_web::{
    delete, get, post,
    web::{Data, Path, Query},
    App, HttpResponse, HttpServer, Responder,
};
#[allow(unused_imports)] // needs to be imported to give method access
use rand::{seq::IteratorRandom, thread_rng};

use serde_derive::Deserialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::{Mutex, RwLock};

type MessageQueueMap = RwLock<HashMap<String, Mutex<Vec<Message>>>>;
type Message = String;
type InactiveUserSet = Mutex<HashSet<String>>;

#[derive(Debug, Deserialize)]
struct PubKeyPair {
    pub_key: String,
    other_pub_key: String,
}

#[post("/users")]
async fn sign_in(inactive_users: Data<InactiveUserSet>, pub_key: String) -> impl Responder {
    let mut users = inactive_users.lock().await;
    users.insert(pub_key);

    HttpResponse::Ok()
}

#[get("/users")]
async fn get_chat_partner(
    inactive_users: Data<InactiveUserSet>,
    pub_key: String,
) -> impl Responder {
    let mut users = inactive_users.lock().await;
    let partner = { users.iter().choose(&mut thread_rng())?.clone() };
    users.remove(&partner);
    users.remove(&pub_key);
    Some(partner)
}

#[post("/messages")]
async fn start_chat(
    active_users: Data<MessageQueueMap>,
    pub_keys: Query<PubKeyPair>,
) -> impl Responder {
    let PubKeyPair {
        pub_key,
        other_pub_key,
    } = pub_keys.into_inner();
    let mut editable_user_map = active_users.write().await;
    editable_user_map.insert(pub_key, Mutex::new(Vec::new()));
    editable_user_map.insert(other_pub_key, Mutex::new(Vec::new()));

    HttpResponse::Ok()
}

// TODO: figure out how other user gets pub key of person matched

#[post("/messages/{pub_key}")]
async fn send_message(
    active_users: Data<MessageQueueMap>,
    pub_key: Path<String>,
    message: String,
) -> impl Responder {
    let user_map = active_users.read().await;
    if let Some(reciever) = user_map.get(pub_key.as_ref()) {
        let mut send_queue = reciever.lock().await;
        send_queue.push(message);
    }
    HttpResponse::Ok()
}

#[get("/messages/{pub_key}")]
async fn get_messages(
    active_users: Data<MessageQueueMap>,
    pub_key: Path<String>,
) -> impl Responder {
    let user_map = active_users.read().await;
    if let Some(user_messages_mutex) = user_map.get(pub_key.as_ref()) {
        let mut messages = user_messages_mutex.lock().await;
        Some(messages.drain(..).collect::<Vec<_>>().join("\n"))
    } else {
        None
    }
}

#[delete("/messages")]
async fn end_chat(
    active_users: Data<MessageQueueMap>,
    pub_keys: Query<PubKeyPair>,
) -> impl Responder {
    let PubKeyPair {
        pub_key,
        other_pub_key,
    } = pub_keys.into_inner();
    let mut editable_user_map = active_users.write().await;
    editable_user_map.remove(&pub_key);
    editable_user_map.remove(&other_pub_key);

    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(sign_in)
            .service(get_chat_partner)
            .service(start_chat)
            .service(send_message)
            .service(get_messages)
    })
    .bind("127.0.0.1:60080")?
    .run()
    .await
}
