use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Folder::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Folder::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(Folder::Path)
              .string()
              .not_null()
              .unique_key(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Folder::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Folder {
  Table,
  Id,
  Path,
}
