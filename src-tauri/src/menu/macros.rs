#[macro_export]
macro_rules! menu_item {
  ($app:expr, $id:expr, $text:expr) => {{
    tauri::menu::MenuItemBuilder::new($text)
      .id($id)
      .build($app)
  }};
  ($app:expr, $id:expr, $text:expr, $accelerator:expr) => {{
    tauri::menu::MenuItemBuilder::new($text)
      .id($id)
      .accelerator($accelerator)
      .build($app)
  }};
}
