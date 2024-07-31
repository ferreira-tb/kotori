pub mod library_book;
pub mod reader_page;

use crate::prelude::*;

#[macro_export]
macro_rules! popup_context_menu {
  ($window:expr, $menu:ident, $ctx:expr) => {{
    if let Some(it) = $window.try_state::<$menu>() {
      it.update(&$ctx)?;
      *it.ctx.lock().unwrap() = $ctx;
      $window.popup_menu(&it.menu)?;
    } else {
      let it = $menu::new($window, $ctx)?;
      $window.popup_menu(&it.menu)?;
      $window.manage(it);
    };

    Ok(())
  }};
}

trait ContextMenuUpdate {
  type Context;

  fn update(&self, _: &Self::Context) -> Result<()> {
    Ok(())
  }
}
