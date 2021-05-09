#![allow(dead_code)]

use rand::{seq::IteratorRandom, thread_rng};
use std::collections::{HashMap, HashSet};
use tokio::sync::{Mutex, RwLock};

const MAX_PENDING_MESSAGES: usize = 16;

type MessageQueueMap = RwLock<HashMap<PubKey, Mutex<Vec<Message>>>>;
type PubKey = [u8; 32];
type Message = Vec<u8>;
type InactiveUserSet = Mutex<HashSet<PubKey>>;

async fn sign_in(inactive_users: InactiveUserSet, pub_key: PubKey) {
    if let Ok(mut users) = inactive_users.lock().await {
        users.insert(pub_key);
    }
}

async fn get_chat_partner(inactive_users: InactiveUserSet, pub_key: PubKey) -> Option<PubKey> {
    if let Ok(mut users) = inactive_users.lock().await {
        let partner = { *users.iter().choose(&mut thread_rng())? };
        users.remove(&partner);
        users.remove(&pub_key);
        Some(partner)
    } else {
        None
    }
}

async fn start_chat(
    active_users: MessageQueueMap,
    user_pub_key: PubKey,
    other_user_pub_key: PubKey,
) {
    if let Ok(mut editable_user_map) = active_users.write().await {
        editable_user_map.insert(user_pub_key, Mutex::new(Vec::new()));
        editable_user_map.insert(other_user_pub_key, Mutex::new(Vec::new()));
    }
}

async fn send_message(active_users: MessageQueueMap, recieving_pub_key: PubKey, message: Message) {
    if let Ok(user_map) = active_users.read().await {
        if let Some(reciever) = user_map.get(&recieving_pub_key) {
            if let Ok(mut send_queue) = reciever.lock().await {
                send_queue.push(message);
            }
        }
    }
}

async fn get_messages(active_users: MessageQueueMap, pub_key: PubKey) -> Option<Vec<Message>> {
    if let Ok(user_map) = active_users.read().await {
        if let Some(user_messages_mutex) = user_map.get(&pub_key) {
            if let Ok(mut messages) = user_messages_mutex.lock().await {
                Some(messages.drain(..).collect())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

async fn end_chat(active_users: MessageQueueMap, user_pub_key: PubKey, other_user_pub_key: PubKey) {
    if let Ok(mut editable_user_map) = active_users.write().await {
        editable_user_map.remove(&user_pub_key);
        editable_user_map.remove(&other_user_pub_key);
    }
}

fn main() {
    println!("Hello, world!");
}
