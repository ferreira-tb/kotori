pub use sea_orm_migration::prelude::*;

mod m20240320_134836_book;
mod m20240513_014639_collection;
mod m20240629_212842_folder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20240320_134836_book::Migration),
      Box::new(m20240513_014639_collection::Migration),
      Box::new(m20240629_212842_folder::Migration),
    ]
  }
}
