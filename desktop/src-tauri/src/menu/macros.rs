#[macro_export]
macro_rules! impl_deref_menu {
  ($($menu:ident),+) => {
    $(
      impl std::ops::Deref for $menu {
        type Target = tauri::menu::Submenu<tauri::Wry>;

        fn deref(&self) -> &Self::Target {
          &self.0
        }
      }
    )+
  };
}

/// Build a menu item.
/// This macro needs an `Item` enum in the scope.
#[macro_export]
macro_rules! mi {
  ($app:expr, $id:ident, $text:expr) => {{
    tauri::menu::MenuItemBuilder::new($text)
      .id(Item::$id)
      .build($app)
  }};
}

#[macro_export]
macro_rules! menu_item_or_bail {
  ($event:expr) => {{
    let Ok(item) = Self::try_from($event.id().as_ref()) else {
      return;
    };

    #[cfg(feature = "tracing")]
    tracing::debug!(menu_event = %item);

    item
  }};
}
