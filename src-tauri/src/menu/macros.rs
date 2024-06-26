#[macro_export]
macro_rules! menu_item {
  ($app:expr, $id:expr, $text:expr) => {{
    tauri::menu::MenuItemBuilder::new($text)
      .id($id)
      .build($app)
  }};
}

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
