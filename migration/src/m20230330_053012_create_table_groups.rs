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
                    .table(Group::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Group::GroupId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Group::GroupName).string().not_null())
                    .col(ColumnDef::new(Group::OwnerId).integer().not_null())
                    .col(ColumnDef::new(Group::GroupCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(Group::GroupUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_group_owner_id")
                    .from_tbl(Group::Table)
                    .from_col(Group::OwnerId)
                    .to_tbl(User::Table)
                    .to_col(User::UserId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await
    }
}

pub enum Group {
    Table,
    GroupId,
    GroupName,
    OwnerId,
    GroupCreatedAt,
    GroupUpdatedAt
}

// Mapping between Enum variant and its corresponding string value
impl Iden for Group {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Groups",
                Self::GroupId => "group_id",
                Self::GroupName => "group_name",
                Self::OwnerId => "owner_id",
                Self::GroupCreatedAt => "created_at",
                Self::GroupUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}