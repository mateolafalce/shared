use axum::response::IntoResponse;
use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use std::{
    fs,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, broadcast};

// TODO impl this as a CLI args
// TODO impl title as a CLI
// TODO just 1 person should be able to share the screen
// TODO add counter of active/waiting users
// TODO add waiting image for client
const PORT: &str = "3000";

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Vec<u8>>(50);
    let tx = Arc::new(Mutex::new(tx));
    let is_sharing = Arc::new(AtomicBool::new(false));

    let app = Router::new()
        .route("/admin", get(serve_index))
        .route("/", get(client_index))
        .route("/ws", get(ws_handler))
        .nest_service(
            "/static",
            axum::routing::get_service(tower_http::services::ServeDir::new("./static")),
        )
        .with_state((tx, is_sharing));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}"))
        .await
        .unwrap();

    println!("In localhost http://127.0.0.1:{PORT}");
    // TODO get 192.168.1.47 in a dynamic way in linux, Win and Mac
    println!("For local network http://192.168.1.47:{PORT}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

macro_rules! serve_html {
    ($func_name:ident, $file_path:expr) => {
        async fn $func_name() -> Html<String> {
            let html = fs::read_to_string($file_path)
                .unwrap_or_else(|_| "<h1>Error</h1>".to_string());
            Html(html)
        }
    };
}

serve_html!(serve_index, "static/index.html");
serve_html!(client_index, "static/client.html");

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((state, is_sharing)): axum::extract::State<(
        Arc<Mutex<broadcast::Sender<Vec<u8>>>>,
        Arc<AtomicBool>,
    )>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, is_sharing))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<Mutex<broadcast::Sender<Vec<u8>>>>,
    is_sharing: Arc<AtomicBool>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut rx = state.lock().await.subscribe();
    let state_clone = state.clone();
    let is_sharing_clone = is_sharing.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Binary(bin) => {
                    if !is_sharing_clone.load(Ordering::SeqCst) {
                        is_sharing_clone.store(true, Ordering::SeqCst);
                    }
                    let _ = state_clone.lock().await.send(bin.to_vec());
                }
                _ => {}
            }
        }
        is_sharing_clone.store(false, Ordering::SeqCst);
    });

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let axum_msg = Message::Binary(msg.into());
            if ws_sender.send(axum_msg).await.is_err() {
                break;
            }
        }
    });
}
