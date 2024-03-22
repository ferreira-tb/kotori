use crate::prelude::*;

pub struct Library;

impl Library {
  pub async fn add<M, R>(_: &M) -> Result<()>
  where
    R: Runtime,
    M: Manager<R>,
  {
    Ok(())
  }
}
