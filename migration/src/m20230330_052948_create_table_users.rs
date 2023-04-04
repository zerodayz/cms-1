use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::UserId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::UserName).string().not_null())
                    .col(ColumnDef::new(User::UserPassword).string().not_null())
                    .col(ColumnDef::new(User::UserToken).string().not_null())
                    .col(ColumnDef::new(User::UserCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(User::UserUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

pub enum User {
    Table,
    UserId,
    UserName,
    UserPassword,
    UserToken,
    UserCreatedAt,
    UserUpdatedAt
}

// Mapping between Enum variant and its corresponding string value
impl Iden for User {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Users",
                Self::UserId => "user_id",
                Self::UserName => "user_name",
                Self::UserPassword => "user_password",
                Self::UserToken => "user_token",
                Self::UserCreatedAt => "created_at",
                Self::UserUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}