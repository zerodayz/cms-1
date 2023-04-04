use sea_orm_migration::prelude::*;
use crate::m20230330_052948_create_table_users::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Space::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Space::SpaceId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Space::SpaceName).string().not_null())
                    .col(ColumnDef::new(Space::OwnerId).integer().not_null())
                    .col(ColumnDef::new(Space::IsPublic).boolean().not_null().default(false))
                    .col(ColumnDef::new(Space::SpaceCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(Space::SpaceUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_space_owner_id")
                    .from_tbl(Space::Table)
                    .from_col(Space::OwnerId)
                    .to_tbl(User::Table)
                    .to_col(User::UserId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Space::Table).to_owned())
            .await
    }
}

pub enum Space {
    Table,
    SpaceId,
    SpaceName,
    OwnerId,
    IsPublic,
    SpaceCreatedAt,
    SpaceUpdatedAt,
}

// Mapping between Enum variant and its corresponding string value
impl Iden for Space {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Spaces",
                Self::SpaceId => "space_id",
                Self::SpaceName => "space_name",
                Self::OwnerId => "owner_id",
                Self::IsPublic => "is_public",
                Self::SpaceCreatedAt => "created_at",
                Self::SpaceUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}