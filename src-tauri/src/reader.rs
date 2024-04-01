use crate::prelude::*;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

pub struct Reader {
  _app: AppHandle,
}

impl Reader {
  pub fn new(app: AppHandle) -> Self {
    Self { _app: app }
  }

  pub fn serve(&self) {
    thread::spawn(|| {
      async_runtime::spawn(async {
        let router = Router::new().route("/", get(root));

        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, router).await.unwrap();
      })
    });
  }
}

async fn root() -> &'static str {
  "Hello, World!"
}
