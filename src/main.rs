#![allow(dead_code)]

use rand::{seq::IteratorRandom, thread_rng};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};

const MAX_PENDING_MESSAGES: usize = 16;

// type RecieveChannel = channel::Receiver<Vec<u8>>;
// type SendChannel = channel::Sender<Vec<u8>>;
// type SendMapWriteHandle = evmap::WriteHandle<[u8; 32], SendChannel>;
// type RecvMapWriteHandle = evmap::WriteHandle<[u8; 32], RecieveChannel>;
// type SendMapReadHandle = evmap::ReadHandle<[u8; 32], SendChannel>;
// type RecvMapReadHandle = evmap::ReadHandle<[u8; 32], RecieveChannel>;

type MessageQueueMap = RwLock<HashMap<PubKey, Mutex<Vec<Message>>>>;
type PubKey = [u8; 32];
type Message = Vec<u8>;
type InactiveUserSet = Mutex<HashSet<PubKey>>;

// fn pair_users(
//     user_send_map: Mutex<SendMapWriteHandle>,
//     user_recv_map: Mutex<RecvMapWriteHandle>,
//     sender_pub_key: [u8; 32],
//     reciever_pub_key: [u8; 32],
// ) {
//     let (message_send_handle, message_recieve_handle) = channel::bounded(MAX_PENDING_MESSAGES);
//     let (other_user_send_handle, other_user_recieve_handle) =
//         channel::bounded(MAX_PENDING_MESSAGES);
//     if let Ok(lock) = user_send_map.lock() {
//         lock.insert(sender_pub_key, message_send_handle);
//         lock.insert(reciever_pub_key, other_user_send_handle);
//     }
//     if let Ok(lock) = user_recv_map.lock() {
//         lock.insert(reciever_pub_key, message_recieve_handle);
//         lock.insert(sender_pub_key, other_user_recieve_handle);
//     }
// }

fn sign_in(inactive_users: InactiveUserSet, pub_key: PubKey) {
    if let Ok(mut users) = inactive_users.lock() {
        users.insert(pub_key);
    }
}

fn get_chat_partner(inactive_users: InactiveUserSet, pub_key: PubKey) -> Option<PubKey> {
    if let Ok(mut users) = inactive_users.lock() {
        let partner = { *users.iter().choose(&mut thread_rng())? };
        users.remove(&partner);
        users.remove(&pub_key);
        Some(partner)
    } else {
        None
    }
}

fn start_chat(active_users: MessageQueueMap, user_pub_key: PubKey, other_user_pub_key: PubKey) {
    if let Ok(mut editable_user_map) = active_users.write() {
        editable_user_map.insert(user_pub_key, Mutex::new(Vec::new()));
        editable_user_map.insert(other_user_pub_key, Mutex::new(Vec::new()));
    }
}

fn send_message(active_users: MessageQueueMap, recieving_pub_key: PubKey, message: Message) {
    if let Ok(user_map) = active_users.read() {
        if let Some(reciever) = user_map.get(&recieving_pub_key) {
            if let Ok(mut send_queue) = reciever.lock() {
                send_queue.push(message);
            }
        }
    }
}

fn get_messages(active_users: MessageQueueMap, pub_key: PubKey) -> Option<Vec<Message>> {
    if let Ok(user_map) = active_users.read() {
        if let Some(user_messages_mutex) = user_map.get(&pub_key) {
            if let Ok(mut messages) = user_messages_mutex.lock() {
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

fn end_chat(active_users: MessageQueueMap, user_pub_key: PubKey, other_user_pub_key: PubKey) {
    if let Ok(mut editable_user_map) = active_users.write() {
        editable_user_map.remove(&user_pub_key);
        editable_user_map.remove(&other_user_pub_key);
    }
}

fn main() {
    println!("Hello, world!");
}
