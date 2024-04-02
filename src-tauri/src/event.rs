use strum::{Display, EnumString};

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Event {
  // AddToLibrary,
  WillMountReader,
}
