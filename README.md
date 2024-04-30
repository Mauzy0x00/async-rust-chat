# Asynchronous Chat Application Written in Rust
#### This repository was a learning project as well as a project for my networking class
#### This program uses Async-std for asynchronous programming and Druid for the UI

# Usage
## Server
- Start the server application
  - That's it.
## Client 
- Start the client application
- Enter a username/alias
- To message another connected client the format is 'recipient: message'
    - For more than one recipient the format is 'recipient1, recipient2, recipient3: message'
- To get a list of connected clients click the "List Clients" button
- The "New Recipient" button is not properly implemented.
    - It will take you to a new window that you cannot return from. Restart the application. 
