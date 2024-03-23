pub enum Event {
  // AddToLibrary,
  OpenBook,
  NavigateToLibrary,
}

impl Event {
  pub fn as_str(&self) -> &str {
    match self {
      // Self::AddToLibrary => "add_to_library",
      Self::NavigateToLibrary => "navigate_to_library",
      Self::OpenBook => "open_book",
    }
  }
}
