/*
    The application UI layout and data links are defined here

    Author: Mauzy0x00
    Date:   3/21/2024
*/

use crate::data::*;

use druid::{ 
    widget::{Button, CrossAxisAlignment, Flex,
            Label, Scroll, SizedBox, TextBox, ViewSwitcher}, Widget, WidgetExt 
};

pub fn build_ui() -> impl Widget<AppState> {

    let view_switcher = ViewSwitcher::new(
        // Selector function to choose the view based on AppState
        |data: &AppState, _env| {
            if data.logged_in && data.current_view == 0 {
                1   // message_ui
            } else {
                // Default to current view (login_ui)
                data.current_view
            }
        },
        // View builder function for each selectable view 
        |selector, _data, _env| match selector{
            0 => Box::new(login_ui()),
            1 => Box::new(chat_ui()),
            2 => Box::new(user_list_ui()),
            _ => Box::new(Label::new("Unknown").center()),
        },
    );

    Flex::row()
        .with_flex_child(view_switcher,1.0)
}

/// Returns a user interface layout for setting the user's alias 
/// TODO: Deny user from entering any special characters such as '**' (** denotes server messages)
pub fn login_ui() -> impl Widget<AppState> {

    // Texbox and send button ==========================================================
    let text_box = TextBox::new()
        .with_placeholder("Username")
        .expand_width()
        .lens(AppState::user_alias)
        .padding(3.0);

    let send_button = Button::new("Send")
        .on_click(move |_ctx, data: &mut AppState, _env| {

            // Get text from the text box and add it to new_user_message
            let message = data.user_alias.clone(); 

            if let Err(err) = data.sender.try_send(message.clone()) {
                eprintln!("Error sending username: {:?}", err);
            } else {
                println!("Username set to: {}", message);
                // Set the user to logged in with the given user alias
                data.logged_in = true;
                data.user_alias = message;
            }

        })
        .padding(3.0);

    // Textbox & send button DIV
    let input_row = Flex::row()
    .with_flex_child(text_box, 1.0)
    .with_spacer(8.0) // Add spacing between text box and button
    .with_child(send_button);
// End Textbox and send button =======================================================
    
    input_row //.debug_paint_layout()
}

/// A user interface that returns a layout for sending and receiving messages
pub fn chat_ui() -> impl Widget<AppState> {

    let message_list: SizedBox<_> = Scroll::new(
        Flex::column()
            .with_flex_child(
                // Display messages
                Label::dynamic(|data: &AppState, _env: &_| {
                    let messages = data
                        .messages
                        .iter()
                        .map(|msg| format!("{}: {} ({})", msg.sender, msg.content, msg.timestamp))
                        .collect::<Vec<String>>()
                        .join("\n");
                    messages
                })
                .padding(8.0)
                .expand_width(),
            1.0)
    )
    .vertical()
    .expand_width();


// Texbox and send button ==========================================================
    let text_box = TextBox::new()
        .with_placeholder("Send message")
        .expand_width()
        .lens(AppState::new_user_message)
        .padding(3.0);


    let send_button = Button::new("Send")
        .on_click(move |_ctx, data: &mut AppState, _env| {

            // Get text from the text box and add it to new_user_message
            let message = data.new_user_message.clone(); // Clone the text to avoid borrowing issues

            // Send the string to the connection Task in main.rs
            // try_send requires error handling
            if let Err(err) = data.sender.try_send(message.clone()) {
                eprintln!("Error sending message: {:?}", err);
            } else {
                println!("Button has been clicked! - Message sent from: {}", message);
            }

            // Set the username to the saved user_alias
            let username: String = data.user_alias.clone();

            // Create a new message
            let new_message = Message {
                sender: String::from(username),
                content: String::from(message),
                timestamp: SystemClock::new_utc().now().format("%Y-%m-%d %H:%M").to_string(),
            };

            // Append the new message to the messages vector
            data.messages.push(new_message);
        })
        .padding(3.0);

    // Textbox & send button DIV
    let input_row = Flex::row()
        .with_flex_child(text_box, 1.0)
        .with_spacer(8.0) // Add spacing between text box and button
        .with_child(send_button);
// End Textbox and send button =======================================================
    
    // Button to switch views to the user list
    let new_recipient_button = Button::new("New Recipient")
        .on_click(move |_ctx, data: &mut AppState, _env| {

            data.current_view = 2;

            // Signal the server for a request for a list of users
            let signal_msg = "Client_PeerList_Request";
            
            if let Err(err) = data.signal_sender.try_send(signal_msg.to_string()) {
                eprintln!("Error sending username: {:?}", err);
            } else {
                println!("Sent server signal");
            }
            
            build_ui();
        })
        .padding(3.0);

        // Button to switch views to the user list
    let list_clients_button = Button::new("List Clients")
        .on_click(move |_ctx, data: &mut AppState, _env| {

            // Signal the server for a request for a list of users
            let signal_msg = "Client_PeerList_Request";
            
            if let Err(err) = data.signal_sender.try_send(signal_msg.to_string()) {
                eprintln!("Error sending username: {:?}", err);
            } else {
                println!("Sent server signal");
            }
            
            build_ui();
        })
        .padding(3.0);


    // Row for client info buttons
    // let client_info = Flex::row()
    //     .with_child(list_clients_button)
    //     .with_child(new_recipient_button);
            
    let layout = Flex::column()
        .with_child(list_clients_button)
        .with_child(new_recipient_button)
        .with_child(Label::new("Chat Messages").padding(8.0).center())
        .with_flex_child(message_list, 1.0)
        .with_child(input_row)
        .cross_axis_alignment(CrossAxisAlignment::End);
    
    layout //.debug_paint_layout()
}


/// A user interface that returns a layout of users currently connected to the server
/// TODO: Make it work
pub fn user_list_ui() -> impl Widget<AppState> {

    // TODO: Populate form with a list of users connected to the server

    let col = Flex::column();
    // let mut row = Flex::row();

    // let check_box = LensWrap::new(Checkbox::new(""), AppState::connected_users);
    
    // row.add_child(Label::new(connected_user.user));
    // row.add_child(Padding::new(5.0, check_box));
    col.center()
}