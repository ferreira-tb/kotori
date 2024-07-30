use super::PageMap;
use crate::book::metadata::Metadata;
use crate::prelude::*;
use crate::utils::result::TxResult;
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
  GetPages {
    path: PathBuf,
    tx: TxResult<Arc<PageMap>>,
  },
  ReadPage {
    path: PathBuf,
    page: String,
    tx: TxResult<Vec<u8>>,
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
  SetMetadata {
    path: PathBuf,
    metadata: Metadata,
    tx: TxResult<()>,
  },
}
