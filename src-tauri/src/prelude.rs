pub use std::path::{Path, PathBuf};

pub use future_iter::prelude::*;
pub use futures::{FutureExt, TryFutureExt};
pub use itertools::Itertools;
pub use tauri::{
  async_runtime::{block_on, spawn, spawn_blocking},
  AppHandle, Manager, WebviewWindow, Window, Wry,
};
pub use tracing::{debug, info, trace, warn};

pub use crate::{
  bail, err,
  error::Result,
  utils::{
    app::AppHandleExt as _,
    path::{PathExt as _, PathResolverExt as _},
    result::ResultExt as _,
  },
};
