/*
    The main function is called here. Depends on data.rs and view.rs 
    Call a task handle the server connection
    New messages from the user's UI will be sent to the server and messages 
    recieved will be sent to the UI to be displayed in chat history.
    
    Author: Mauzy0x00
    Date:   3/21/2024

*/

use druid::{AppLauncher, WindowDesc};

mod data;
use data::{AppState, Message, SystemClock};
use crate::data::*;

mod view;
use view::build_ui;

use futures::{select, FutureExt};

use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
    task,
    channel::{unbounded,  Sender, Receiver}
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub(crate) fn main() -> Result<()> {

    // Create an unbounded channel to send messages from build_ui to main
    let (sender, receiver) = unbounded::<String>(); // Specify type <T> as String

    // Create an unbounded channel to recieve a list of users from the server
    let (signal_sender, signal_reciever) = unbounded::<String>();

    // Setup UI
    let main_window = WindowDesc::new(build_ui())
        .title("Mauzy's Rusty Chat App")
        .window_size((400.0, 300.0));

    let launcher = AppLauncher::with_window(main_window);

    // If we want to create commands from another thread `launcher.get_external_handle()`
    // should be used. For sending commands from within widgets you can always call
    // `ctx.submit_command`
    let event_sink = launcher.get_external_handle();

    // Run the try_run task
    task::spawn(connection("127.0.0.1:1632", receiver, signal_reciever, event_sink));

    // Run the UI in the main thread
    user_interface(launcher, sender, signal_sender);

    Ok(())
}


async fn connection(addr: impl ToSocketAddrs, receiver: Receiver<String>, signal_reciever: Receiver::<String>, event_sink: druid::ExtEventSink) -> Result<()> {
    
    // Connect to the server
    // Hold the code here; 'await' until a connection is made
    println!("Connecting to server...\n");
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = (&stream, &stream);
    println!("Connected to server!");

    // Set up a buffered reader to reit worksad lines from the server
    let reader = BufReader::new(reader);
    let mut lines_from_server = futures::StreamExt::fuse(reader.lines());


    // Start an event loop to handle incoming messages from the server and user input
    loop {
        select! {
            // Read lines from the server socket
            // Receive messages from the server and send to UI
            server_message = lines_from_server.next().fuse() => match server_message {
                Some(server_message) => {
                    let server_message = server_message?;

                    let message_check = server_message.clone();

                    if message_check == "**Client_list"     // Dead
                    {   

                        // Recieve client list until the end
                        let sig_fin: bool = false;
                        while !sig_fin 
                        {
                            // TODO: Read lines from server and fill a vector
                        }


                    } else {
                        // schedule idle callback to change the data
                        event_sink.add_idle_callback(move |data: &mut AppState| {
                            let message = server_message.clone();
                                
                            // Split the string by ": " to separate the components
                            let parts: Vec<&str> = message.split(": ").collect();

                            let username = parts[0].trim();
                            let message = parts[1..].join(": ");

                            // If the message is from the server indicating a new user, add it to the connected user list
                            if message.starts_with("**New User Connected:") {
                                let new_connected_user = ConnectedUsers {
                                    user: String::from(username),
                                    selected: false
                                };
                                data.connected_users.push(new_connected_user);
                            }

                            // terminal logging
                            println!("username {}", username);  
                            println!("message {},", message);

                            // Temp code to make client listing prettier 
                            if username == "**Server" || username == "**FIN" {
                                let server_message = Message {
                                    sender: String::from(username),
                                    content: String::from(message),
                                    timestamp: String::from(""),
                                };
                                data.messages.push(server_message);

                            } else {
                                // Create a new message
                                let new_message = Message {
                                    sender: String::from(username),
                                    content: String::from(message),
                                    timestamp: SystemClock::new_utc().now().format("%H:%M %Y-%m-%d").to_string(),
                                };
                                data.messages.push(new_message);
                            }
                        });
                    }

                }
                None => {
                    println!("Channel closed, exiting event loop");
                    break; // Break if the channel is closed
                }
            },

            // Receive messages from the UI
            ui_message = receiver.recv().fuse() => match ui_message {
                Ok(user_text) => {
                    // Write the user message to the server
                    writer.write_all(user_text.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    println!("recieved from UI: {}", user_text);
            
                }
                Err(_) => {
                    println!("Channel closed, exiting event loop.");
                    break; // Break if the channel is closed
                }
            },
            // Receive signals from the UI to the connection thread to send requests to the server
            signal = signal_reciever.recv().fuse() => match signal {
                Ok(signal) => {
                    // Write the user message to the server
                    writer.write_all(signal.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    println!("recieved from UI: {}", signal);
                }
                Err(_) => {
                    println!("Signal channel closed, exiting event loop.");
                    break; // Break if the signal channel is closed
                }
            }
        }
    }
    
    // Write the disconnect message to the server
    let disconnect_msg = "Client_Disconnect";
    writer.write_all(disconnect_msg.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    
    Ok(())
}

/// Function to launch the application 
fn user_interface(launcher: AppLauncher<AppState>, sender: Sender<String>, signal_sender: Sender<String>) {

    // Initialize the app state
    let initial_state = AppState {
        current_view: 0,

        logged_in: false,
        user_alias: String::new(),
        new_user_message: String::new(),
        new_socket_message: String::new(),
        messages: Vec::new(),   
        connected_users: Vec::new(),
        
        sender: sender, 
        signal_sender: signal_sender
    };


    // Start the application
    launcher
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}
