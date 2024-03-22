use crate::prelude::*;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Metadata {
  pub path: PathBuf,
  pub title: String,
}
