use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::routing::post;

mod json_converter;
mod data_manipulate;
mod request_with_json_file;

#[tokio::main]
async  fn main() {

    let app = Router::new()
        .route("manipulate-datas", post(request_with_json_file::convert_to_json_string))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
