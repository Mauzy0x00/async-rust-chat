/*
    Author: Mauzy0x00
    Date:   3/21/2024

    This Rust code implements a simple peer-to-peer network using asynchronous I/O and channels for message passing.
    The `accept_loop` function asynchronously accepts incoming TCP connections on the specified address, spawning connection tasks for each accepted connection and managing a broker loop for handling peer connections and messages.
    The `connection_loop` function handles communication with a client, forwarding messages to the broker and notifying it about new peer connections.
    The `connection_writer_loop` function continuously writes messages from a channel to a TCP stream, listening for a shutdown signal to exit gracefully.
    The `broker_loop` function is an asynchronous event loop for managing peer connections and message forwarding, with support for disconnecting peers and cleanup.
    The code uses the `futures` and `async_std` crates for asynchronous programming, and it defines custom event types to represent different actions within the peer-to-peer network.
    Note: The code includes error handling and logging for any encountered errors.

*/
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
};

use futures::{channel::mpsc, select, FutureExt, SinkExt};

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug)]
enum Void {}

fn main() -> Result<()> {
    task::block_on(accept_loop("127.0.0.1:1632"))
}

/// Asynchronously accepts incoming TCP connections on the specified address,
/// spawns connection tasks for each accepted connection, and manages a broker loop
/// for handling peer connections and messages.
async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let (broker_sender, broker_receiver) = mpsc::unbounded();
    let broker = task::spawn(broker_loop(broker_receiver));
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        spawn_and_log_error(connection_loop(broker_sender.clone(), stream));
    }
    drop(broker_sender);
    broker.await;
    Ok(())
}

/// Asynchronous function to handle communication with a client,
/// forwarding messages to the broker and notifying it about new peer connections.
async fn connection_loop(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let stream = Arc::new(stream);
    let reader = BufReader::new(&*stream);
    let mut lines = reader.lines();

    // set the username of the client 
    let name = match lines.next().await {
        None => return Err("peer disconnected immediately".into()),
        Some(line) => line?,
    };

    let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
    // Send a message to the broker about a new peer 
    broker
        .send(Event::NewPeer {
            name: name.clone(),
            stream: Arc::clone(&stream),
            shutdown: shutdown_receiver,
        })
        .await
        .unwrap();

    // Send a notification about the new client to all existing clients
    broker
        .send(Event::Message {
            from: "**".to_string(),         // Use Server indicates a system message, not user
            to: vec!["*".to_string()],          // Send to all clients ("*" represents all)
            msg: format!("New client joined: {}", name),
        })
        .await
        .unwrap();


    // Get the lines read in from the client 
    while let Some(line) = lines.next().await {
        let line = line?;

        println!("Client msg: {}", line);
        // If a client sends a disconnect signal
        if line == "Client_Disconnect" {
            broker 
                .send(Event::Message { 
                    from: "**".to_string(),                 // Use Server indicates a system message, not user
                    to: vec!["*".to_string()],              // Send to all clients ("*" represents all)
                    msg: format!("Client, {}, has disconnected ", name),
                })
                .await
                .unwrap();
        }

        if line == "Client_PeerList_Request" {
            broker
                .send(Event::ClientListRequest { 
                    from: name.to_string(),
                })
                .await
                .unwrap()
        }
        
        let (dest, msg) = match line.find(':') {
            None => continue,
            Some(idx) => (&line[..idx], line[idx + 1..].trim()),
        };

        let dest: Vec<String> = dest
            .split(',')
            .map(|name| name.trim().to_string())
            .collect();
        let msg: String = msg.trim().to_string();

        broker
            .send(Event::Message {
                from: name.clone(),
                to: dest,
                msg,
            })
            .await
            .unwrap();
    }

    Ok(())
}

/// Asynchronous function to continuously write messages from a channel to a TCP stream,
/// listening for a shutdown signal to exit gracefully.
async fn connection_writer_loop(
    messages: &mut Receiver<String>,
    stream: Arc<TcpStream>,
    mut shutdown: Receiver<Void>,
) -> Result<()> {
    let mut stream = &*stream;
    loop {
        select! {
            msg = messages.next().fuse() => match msg {
                Some(msg) => stream.write_all(msg.as_bytes()).await?,
                None => break,
            },
            void = shutdown.next().fuse() => match void {
                Some(void) => match void {},
                None => break,
            }
        }
    }
    Ok(())
}

/// Represents events in the network
#[derive(Debug)]
enum Event {
    // Indicates a new peer connection with the given name, TCP stream, and shutdown receiver.
    NewPeer {
        name: String,
        stream: Arc<TcpStream>,
        shutdown: Receiver<Void>,
    },
    // Indicates a message sent from one peer to one or more destination peers.
    Message {
        from: String,
        to: Vec<String>,
        msg: String,
    },
    // Indicates a client is requesting a list of the connected users.
    ClientListRequest {
        from: String,
    }
}

/// Asynchronous event loop for managing peer connections and message forwarding,
/// with support for disconnecting peers and cleanup.
async fn broker_loop(mut events: Receiver<Event>) {
    // Channel for notifying about peer disconnection (name and pending messages)
    let (disconnect_sender, mut disconnect_receiver) = mpsc::unbounded::<(String, Receiver<String>)>();

    // HashMap to store connected peers (name -> message sender)
    // Hashmap contains the user's chosen name as the key and the unbounded mpsc channel 'client_sender'
    let mut peers: HashMap<String, Sender<String>> = HashMap::new();

    loop {
        // Wait for either an event from the main loop or a disconnect notification
        let event = select! {
            event = events.next().fuse() => match event {
                None => break,
                Some(event) => event,
            },

            disconnect = disconnect_receiver.next().fuse() => {
                let (name, _pending_messages) = disconnect.unwrap();
                assert!(peers.remove(&name).is_some());

                continue;
            },
        };

        match event {
            
            Event::Message { from, to, msg } => {
                // Handle incoming message: send to intended recipients
                if to == vec!["*".to_string()] {
                    // Send to all clients
                    // `HashMap::iter()` returns an iterator that yields 
                    // (&'a key, &'a value) pairs in arbitrary order.
                    for (_name, client_sender_channel) in &peers {
                            let mut peer = client_sender_channel;
                            let msg = format!("{}{}\n", from, msg);
                            peer.send(msg).await.unwrap();
                    }
                } else {
                    for addr in to {
                        // Check if the name is in the hashtable
                        if let Some(peer) = peers.get_mut(&addr) {
                            let msg = format!("{}: {}\n", from, msg);
                            peer.send(msg).await.unwrap();
                        }
                    }
                }
            },

            Event::NewPeer { name, stream, shutdown } => match peers.entry(name.clone()) {
                // Handle new peer connection:
                Entry::Occupied(..) => (),          // Ignore duplicate connection attempts
                Entry::Vacant(entry) => {
                    // Create a new channel for sending messages to this peer
                    let (client_sender, mut client_receiver) = mpsc::unbounded();
                    entry.insert(client_sender);
                
                    // Spawn a separate task to handle writing messages to the peer
                    let mut disconnect_sender = disconnect_sender.clone();
                    spawn_and_log_error(async move {
                        let res = connection_writer_loop(&mut client_receiver, stream, shutdown).await;
                        disconnect_sender
                            .send((name, client_receiver))
                            .await
                            .unwrap();
                        res
                    });
                }
            },
            
            Event::ClientListRequest { from } => {
                // Collect all names from the hashmap into a vector
                let names: Vec<_> = peers.keys().cloned().collect();

                // The client that sent the request recieves the list
                // Make sure the client is in the hashtable 
                if let Some(peer) = peers.get_mut(&from) {

                    let start_msg = format!("**Clients Connected:\n");
                    peer.send(start_msg).await.unwrap();

                    // Iterate over the vector and send each name followed by "FIN"
                    for name in names {
                        // Get rid of the ':'
                        let formated_name = name.trim_end_matches(':').to_string();
                        // Send name
                        let msg = format!("**Server: {}\n", formated_name);
                        peer.send(msg).await.unwrap();
                    }
                    // Send "**FIN" to denote end of list. Don't allow ** char in username
                    let fin_msg = format!("**FIN\n");
                    peer.send(fin_msg).await.unwrap();
                }
            },
        } 
    }
    drop(peers);
    drop(disconnect_sender);
    while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {}
}

/// Spawns a new asynchronous task to execute the given future, logging any errors that occur.
fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}