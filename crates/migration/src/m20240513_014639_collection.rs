use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Collection::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Collection::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(Collection::Name)
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
      .drop_table(Table::drop().table(Collection::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Collection {
  Table,
  Id,
  Name,
}
