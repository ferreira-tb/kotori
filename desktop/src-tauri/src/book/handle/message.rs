use super::PageMap;
use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::result::TxResult;
use std::fmt;
use std::sync::Arc;
use strum::Display;
use tokio::sync::Notify;

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub(super) enum Message {
  Close {
    path: PathBuf,
    nt: Arc<Notify>,
  },
  DeletePage {
    path: PathBuf,
    page: String,
    tx: TxResult<()>,
  },
  GetFirstPageName {
    path: PathBuf,
    tx: TxResult<String>,
  },
  GetMetadata {
    path: PathBuf,
    tx: TxResult<Option<Metadata>>,
  },
  GetPages {
    path: PathBuf,
    tx: TxResult<Arc<PageMap>>,
  },
  ReadPage {
    path: PathBuf,
    page: String,
    tx: TxResult<Vec<u8>>,
  },
  SetMetadata {
    path: PathBuf,
    metadata: Metadata,
    tx: TxResult<()>,
  },
}

impl fmt::Debug for Message {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Message")
      .field(&self.to_string())
      .finish()
  }
}
