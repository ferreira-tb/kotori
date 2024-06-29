use crate::book::Title;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
  pub title: Option<Title>,
  pub rating: Option<i32>,
  pub cover: Option<String>,
}
