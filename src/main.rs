use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::routing::post;

mod json_converter;
mod data_manipulate;
mod request_with_json_file;

#[tokio::main]
async  fn main() {

    let app = Router::new()
        .route("/manipulate-datas", post(request_with_json_file::convert_to_json_string))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024));

    // Inside a container the loopback default is unreachable from the host, so
    // the image sets BIND_ADDR=0.0.0.0:3001.
    let bind_addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3001".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
