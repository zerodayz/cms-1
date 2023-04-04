use sea_orm_migration::prelude::*;
use crate::m20230330_052948_create_table_users::User;
use crate::m20230330_053012_create_table_groups::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupUsers::UserId).integer().not_null())
                    .col(ColumnDef::new(GroupUsers::GroupId).integer().not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_groups_users_user_id")
                    .from_tbl(GroupUsers::Table)
                    .from_col(GroupUsers::UserId)
                    .to_tbl(User::Table)
                    .to_col(User::UserId)
                    .to_owned(),
            ).await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_groups_users_group_id")
                    .from_tbl(GroupUsers::Table)
                    .from_col(GroupUsers::GroupId)
                    .to_tbl(Group::Table)
                    .to_col(Group::GroupId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupUsers::Table).to_owned())
            .await
    }
}

enum GroupUsers {
    Table,
    Id,
    UserId,
    GroupId
}

// Mapping between Enum variant and its corresponding string value
impl Iden for GroupUsers {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Groups_Users",
                Self::Id => "id",
                Self::UserId => "user_id",
                Self::GroupId => "group_id",
            }
        )
            .unwrap();
    }
}