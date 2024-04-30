/*
    The data structures of the application GUI are defined here
    
    Author: Mauzy0x00
    Date:   3/21/2024

    NOTES:
    Clone: This trait allows instances of the struct to be cloned, creating a new instance with the same values. 
    PartialEq: This trait allows instances of the struct to be compared for equality using the == operator. It's useful for determining if the state has changed and needs to be updated in the UI.
    Data: This trait is used to mark types that can be efficiently compared for equality and hashed. This is crucial for optimizing updates in the UI by determining if a part of the state has changed.
    Lens: This trait is used to define how to access and modify nested fields within a struct. Lenses provide a way to update nested data structures in an ergonomic and composable manner.
*/

use async_std::channel::Sender;
use druid::{Data, Lens};
use chrono::{DateTime, TimeZone, Utc};
//use std::time::SystemTime;

// Define a struct to represent the application state
#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub current_view: u32,                  // Unsigned integer for the view selection

    pub logged_in: bool,                    // Bool value to check if the user is logged in or not
    pub user_alias: String,                 // Store the user's chosen username 
    pub new_user_message: String,
    pub new_socket_message: String,

    #[data(eq)]
    pub connected_users: Vec<ConnectedUsers>,    // Store a dynamic list of connected users 

    #[data(eq)]
    pub messages: Vec<Message>,             // Store all of the messages 
    
    #[data(ignore)]
    pub sender: Sender<String>,              // Store the channel sender to communicate between threads 
    #[data(ignore)]
    pub signal_sender: Sender<String>        // Store the channel signal_sender to communicate between threads 
}


// Define a struct to represent a chat message
#[derive(Clone, PartialEq, Data, Lens)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: String
}

#[derive(Clone, PartialEq, Data, Lens)]
pub struct ConnectedUsers {
    pub user: String, 
    pub selected: bool               // Store if the user is selected in the dm pane
}


/// Time data  =============================
pub trait Clock<Tz: TimeZone> {
    fn now(&self) -> DateTime<Tz>;
}

pub struct SystemClock<Tz: TimeZone> {
    time_zone: Tz,
}

impl SystemClock<Utc> {
    pub fn new_utc() -> SystemClock<Utc> {
        SystemClock { time_zone: Utc }
    }
}

/// Dead code
/// TODO: Implement Local Time
// impl<Tz: TimeZone> SystemClock<Tz> {
//     pub fn new_with_time_zone(tz: Tz) -> SystemClock<Tz> {
//         SystemClock { time_zone: tz }
//     }
// }

impl<Tz: TimeZone> Clock<Tz> for SystemClock<Tz> {
    fn now(&self) -> DateTime<Tz> {
        Utc::now().with_timezone(&self.time_zone)
    }
}

/* Example usage
fn main() {
    println!("{:?}", SystemClock::new_utc().now());
    println!("{:?}", SystemClock::new_with_time_zone(FixedOffset::east(1)).now());
    // ...
}
 */
// ===============================================================