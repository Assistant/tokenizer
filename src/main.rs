use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use directories::ProjectDirs;
use get_user_input::get_input;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::io;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::net::TcpListener;

type StateType = State<(String, String, String, Client)>;
type QueryType = Query<HashMap<String, String>>;

static PROJECT_DIR: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("moe", "assistant", "tokenizer").expect("Couldn't get project directory.")
});

#[tokio::main]
async fn main() {
    let config_dir = PROJECT_DIR.config_local_dir();

    if !config_dir.exists() {
        create_dir_all(config_dir).expect(&format!(
            "Couldn't create project config directory: {config_dir:?}"
        ));
    }

    let id = get_value(config_dir.join("client_id.txt"), "Client ID");
    let secret = get_value(config_dir.join("client_secret.txt"), "Client Secret");
    let scopes = get_value(config_dir.join("scopes.txt"), "Scopes");

    let client = Client::new();
    let state = (id.clone(), secret.clone(), scopes, client.clone());

    refresh(&id, &secret, config_dir.join("refresh_token.txt"), client).await;

    let app = Router::new().route("/", get(root)).with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    _ = open::that("http://localhost:3000/");
    axum::serve(listener, app).await.unwrap();
}

async fn root(State((id, secret, scope, client)): StateType, Query(params): QueryType) -> Response {
    if params.is_empty() {
        return index(&id, &scope);
    }

    if let Some(code) = params.get("code") {
        let body = [
            ("grant_type", "authorization_code"),
            ("client_id", id.as_str()),
            ("client_secret", secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", "http://localhost:3000"),
        ];

        let Ok(request) = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&body)
            .send()
            .await
        else {
            return message("Request Failed", "Failed to send request.");
        };

        let Ok(tokens) = request.json::<TokenResponse>().await else {
            return message("Request Failed", "Failed to parse response.");
        };

        let refresh_token_path = PROJECT_DIR.config_local_dir().join("refresh_token.txt");
        _ = trim_write(&refresh_token_path, &tokens.refresh_token);
        return success(&tokens.access_token, &tokens.refresh_token);
    }

    if let Some(error) = params.get("error") {
        if let Some(error_description) = params.get("error_description") {
            return message(error, error_description);
        }
        return message(error, "no description");
    }

    message("Something went wrong", "dunno what")
}

fn get_value(path: PathBuf, msg: &str) -> String {
    if let Ok(value) = trim_read(&path) {
        println!("{msg} found at: {path:?}");
        value
    } else {
        println!("Please enter your {msg}:");
        let input = get_input!(String).expect("Failed to read input.");
        trim_write(&path, &input).expect(&format!("Failed to write {msg}"));
        input
    }
}

fn index(id: &str, scope: &str) -> Response {
    Html::from(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="color-scheme" content="light dark"><title>Authorize</title></head><body align="center"><div role="main" align="center"><a href="https://id.twitch.tv/oauth2/authorize?client_id={id}&redirect_uri=http://localhost:3000&response_type=code&scope={scope}">Authorize</a></div></body></html>"#
    )).into_response()
}

fn message(title: &str, msg: &str) -> Response {
    Html::from(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="color-scheme" content="light dark"><title>{title}</title></head><body align="center"><div role="main" align="center"><h1>{title}</h1><p>{msg}</p></div></body></html>"#
    )).into_response()
}

fn success(access_token: &str, refresh_token: &str) -> Response {
    message("Success!", &format!("access_token: <kbd>{access_token}</kbd></p><p><kbd>oauth:{access_token}</kbd></p><p>refresh_token: <kbd>{refresh_token}</kbd>"))
}

fn trim_read(path: &PathBuf) -> io::Result<String> {
    read_to_string(path).map(|s| s.trim().to_owned())
}

fn trim_write(path: &PathBuf, content: &str) -> io::Result<()> {
    write(path, content.trim())
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    scope: Vec<String>,
    token_type: String,
}

async fn refresh(id: &str, secret: &str, path: PathBuf, client: Client) {
    if let Ok(refresh_token) = trim_read(&path) {
        let body = [
            ("grant_type", "refresh_token"),
            ("client_id", id),
            ("client_secret", secret),
            ("refresh_token", refresh_token.as_str()),
        ];

        let Ok(request) = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&body)
            .send()
            .await
        else {
            return;
        };

        let Ok(tokens) = request.json::<TokenResponse>().await else {
            return;
        };

        println!(
            "\n\nToken refreshed: {token}\noauth:{token}",
            token = tokens.access_token
        );

        _ = trim_write(&path, &tokens.refresh_token);

        loop {}
    }
}
