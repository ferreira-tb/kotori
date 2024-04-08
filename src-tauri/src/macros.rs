#[macro_export]
macro_rules! windows {
  ($app:expr) => {{
    let kotori = $app.state::<$crate::Kotori>();
    let reader = kotori.reader.read().await;
    reader.windows()
  }};
}
