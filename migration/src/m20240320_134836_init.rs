use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Book::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Book::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(Book::Path)
              .string()
              .not_null()
              .unique_key(),
          )
          .col(
            ColumnDef::new(Book::Rating)
              .integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(Book::Cover).string())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Book::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Book {
  Table,
  Id,
  Rating,
  Path,
  Cover,
}
