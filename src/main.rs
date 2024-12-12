use tokio::sync::broadcast;
use warp::Filter;
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    // Create a channel for message broadcasting
    let (tx, _rx) = broadcast::channel(100);

    // Define the WebSocket route
    let signaling_route = warp::path("signaling")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx.clone();
            ws.on_upgrade(move |websocket| handle_signaling(websocket, tx))
        });

    // Start the server
    println!("Signaling server running on ws://127.0.0.1:3030/signaling");
    warp::serve(signaling_route).run(([127, 0, 0, 1], 3030)).await;
}

// Handle WebSocket connections
async fn handle_signaling(ws: warp::ws::WebSocket, tx: broadcast::Sender<String>) {
    use futures_util::{StreamExt, SinkExt}; // Import the traits

    // Split WebSocket into a sender and receiver
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let mut tx_subscriber = tx.subscribe();

    // Task for receiving messages from the WebSocket
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(message)) = user_ws_rx.next().await {
            if let Ok(text) = message.to_str() {
                tx.send(text.to_string()).unwrap();
            }
        }
    });

    // Task for sending messages to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(message) = tx_subscriber.recv().await {
            if user_ws_tx.send(warp::ws::Message::text(message)).await.is_err() {
                break;
            }
        }
    });

    // Wait for both tasks to complete
    tokio::join!(receive_task, send_task);
}
