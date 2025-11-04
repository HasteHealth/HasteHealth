use std::path::PathBuf;

use axum::{
    Router,
    http::{StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
    routing,
};
use rust_embed::Embed;

pub fn root_asset_route() -> PathBuf {
    ["/assets", SERVER_VERSION].iter().collect()
}

pub fn asset_route(asset: &str) -> String {
    let path = root_asset_route();
    path.join(asset).to_str().unwrap().to_string()
}

static SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Embed)]
#[folder = "public/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

// We use a wildcard matcher ("/dist/*file") to match against everything
// within our defined assets directory. This is the directory on our Asset
// struct below, where folder = "examples/public/".
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    let root_path = root_asset_route();
    let root = root_path.to_str().unwrap_or_default();
    if path.starts_with(root) {
        path = path.replace(root, "");
    }

    StaticFile(path)
}

// Finally, we use a fallback route for anything that didn't match.
async fn not_found() -> Html<&'static str> {
    Html("<h1>404</h1><p>Not Found</p>")
}

pub fn create_static_server() -> Router {
    let assets_router = Router::new()
        .route("/", routing::get(index_handler))
        .route("/index.html", routing::get(index_handler))
        .route("/{*file}", routing::get(static_handler))
        .fallback_service(routing::get(not_found));
    assets_router
}
