pub use std::path::{Path, PathBuf};

pub use future_iter::prelude::*;
pub use futures::{FutureExt, TryFutureExt};
pub use itertools::Itertools;
pub use tauri::async_runtime::{block_on, spawn, spawn_blocking};
pub use tauri::{AppHandle, Manager, WebviewWindow, Window, Wry};
pub use tracing::{debug, info, trace, warn};

pub use crate::error::Result;
pub use crate::utils::app::AppHandleExt as _;
pub use crate::utils::path::{PathExt as _, PathResolverExt as _};
pub use crate::utils::result::ResultExt as _;
pub use crate::{bail, err};
