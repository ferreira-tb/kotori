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

/// Try to convert the event id to a menu item variant.
/// If it fails, return early from the listener.
#[macro_export]
macro_rules! menu_item_or_bail {
  ($event:expr) => {{
    let Ok(item) = Self::try_from($event.id().as_ref()) else {
      return;
    };

    tracing::debug!(menu_event = %item);
    item
  }};
}
