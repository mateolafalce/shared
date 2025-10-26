use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::{
    env::consts::OS,
    net::SocketAddr,
    process::{Command, exit},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, broadcast};

/// Admin html
const ADMIN: &str = include_str!("../static/index.html");
/// Client html
const CLIENT: &str = include_str!("../static/client.html");
/// Icon
const FAVICON: &[u8] = include_bytes!("../static/icon.ico");
/// Waiting image
const WAITING: &[u8] = include_bytes!("../static/waiting.png");
/// Default port
const PORT: u16 = 3000;
/// Default title
const TITLE: &str = "shared";

#[derive(Clone)]
struct AppState {
    state: Arc<Mutex<broadcast::Sender<Vec<u8>>>>,
    is_sharing: Arc<AtomicBool>,
    title: String,
    port: u16,
}

impl AppState {
    fn new(title: String, port: u16) -> Self {
        let (tx, _) = broadcast::channel::<Vec<u8>>(50);
        let state = Arc::new(Mutex::new(tx));
        let is_sharing = Arc::new(AtomicBool::new(false));
        Self {
            state,
            is_sharing,
            title,
            port,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port you want the program to run on, by default 3000
    #[arg(short, long)]
    port: Option<u16>,
    /// The title of the page, by default "shared"
    #[arg(short, long)]
    title: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut port: u16 = match args.port {
        Some(port) => port,
        None => PORT,
    };

    let title: String = match args.title {
        Some(title) => title,
        None => TITLE.to_string(),
    };

    let state = AppState::new(title, port);

    let app = Router::new()
        .route("/admin", get(serve_index))
        .route("/", get(client_index))
        .route("/ws", get(ws_handler))
        .route("/icon.ico", get(get_favicon))
        .route("/waiting.png", get(get_waiting))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

macro_rules! serve_html {
    ($func_name:ident, $const_name:expr) => {
        async fn $func_name(
            axum::extract::State(state): axum::extract::State<AppState>,
        ) -> Html<String> {
            let mut html = String::from($const_name);
            html.push_str(&format!("<title>{}</title>", state.title));
            let urls = get_urls(state.port);
            html.push_str(&urls.0);
            html.push_str(&urls.1);
            html.push_str(&urls.2);
            Html(html)
        }
    };
}

serve_html!(serve_index, ADMIN);
serve_html!(client_index, CLIENT);

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.state, state.is_sharing))
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
        let _ = ws_sender
            .send(Message::Text(
                r#"{"type":"sharing_status","status":"busy"}"#.into(),
            ))
            .await;
    } else {
        let _ = ws_sender
            .send(Message::Text(
                r#"{"type":"sharing_status","status":"available"}"#.into(),
            ))
            .await;
    }

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Binary(bin) = msg {
                if !is_sharing_clone.load(Ordering::SeqCst) {
                    is_sharing_clone.store(true, Ordering::SeqCst);
                }
                let _ = state_clone.lock().await.send(bin.to_vec());
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

fn get_urls(port: u16) -> (String, String, String) {
    let local = format!("<p style=\"margin: 10px\">Users in localhost http://127.0.0.1:{port}</p>");
    let admin =
        format!("<p style=\"margin: 10px\">Admin in localhost http://127.0.0.1:{port}/admin</p>");
    let mut network = String::new();
    // TODO get local network ip in Win and Mac
    if cfg!(target_os = "linux") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("hostname -I | awk '{print $1}'")
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let ip = String::from_utf8_lossy(&output.stdout);
            network = format!(
                "<p style=\"margin: 10px\">Users in local network http://{}:{port}</p>",
                ip.trim()
            );
        }
    }
    (local, admin, network)
}

async fn get_favicon() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/x-icon")],
        FAVICON,
    )
        .into_response()
}

async fn get_waiting() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/x-icon")],
        WAITING,
    )
        .into_response()
}
