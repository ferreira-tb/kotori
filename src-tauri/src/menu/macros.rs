
#[macro_export]
macro_rules! menu_item {
  ($app:expr, $id:expr, $text:expr) => {{
    tauri::menu::MenuItemBuilder::with_id($id, $text).build($app)
  }};
  ($app:expr, $id:expr, $text:expr, $accelerator:expr) => {{
    tauri::menu::MenuItemBuilder::with_id($id, $text)
      .accelerator($accelerator)
      .build($app)
  }};
}