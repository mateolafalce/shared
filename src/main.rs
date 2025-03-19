use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::{
    fs,process::Command,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, broadcast};

/// Default port
const PORT: &str = "3000";
/// Default title
const TITLE: &str = "shared";

#[derive(Clone)]
struct AppState {
    state: Arc<Mutex<broadcast::Sender<Vec<u8>>>>,
    is_sharing: Arc<AtomicBool>,
    title: String,
}

impl AppState {
    fn new(title: String) -> Self {
        let (tx, _) = broadcast::channel::<Vec<u8>>(50);
        let state = Arc::new(Mutex::new(tx));
        let is_sharing = Arc::new(AtomicBool::new(false));
        Self {
            state,
            is_sharing,
            title,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The maximum number of messages you want to store
    #[arg(short, long)]
    port: Option<String>,
    /// The cost of the Proof of Work that will be sent to the client
    #[arg(short, long)]
    title: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let port: String = match args.port {
        Some(port) => port,
        None => PORT.to_string(),
    };

    let title: String = match args.title {
        Some(title) => title,
        None => TITLE.to_string(),
    };
    let state = AppState::new(title);

    let app = Router::new()
        .route("/admin", get(serve_index))
        .route("/", get(client_index))
        .route("/ws", get(ws_handler))
        .nest_service(
            "/static",
            axum::routing::get_service(tower_http::services::ServeDir::new("./static")),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Users in localhost http://127.0.0.1:{port}");
    println!("Admin in localhost http://127.0.0.1:{port}/admin");
    // TODO get local network ip in Win and Mac
    if cfg!(target_os = "linux") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("hostname -I | awk '{print $1}'")
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let ip = String::from_utf8_lossy(&output.stdout);
            println!("Users in local network http://{}:{port}", ip.trim());
        } else {
            eprintln!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

macro_rules! serve_html {
    ($func_name:ident, $file_path:expr) => {
        async fn $func_name(
            axum::extract::State(state): axum::extract::State<AppState>,
        ) -> Html<String> {
            let mut html =
                fs::read_to_string($file_path).unwrap_or_else(|_| "<h1>Error</h1>".to_string());
            html.push_str(&format!("<title>{}</title>", state.title));
            Html(html)
        }
    };
}

serve_html!(serve_index, "static/index.html");
serve_html!(client_index, "static/client.html");

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            state.state,
            state.is_sharing,
        )
    })
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

    if is_sharing_clone.load(Ordering::SeqCst) {
        let _ = ws_sender.send(Message::Text(r#"{"type":"sharing_status","status":"busy"}"#.into())).await;
    } else {
        let _ = ws_sender.send(Message::Text(r#"{"type":"sharing_status","status":"available"}"#.into())).await;
    }

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
