use sea_orm_migration::prelude::*;
use crate::m20230330_053017_create_table_spaces::Space;
use crate::m20230330_053012_create_table_groups::Group;
use crate::m20230330_053015_create_table_roles::Role;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupSpaces::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupSpaces::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupSpaces::GroupId).integer().not_null())
                    .col(ColumnDef::new(GroupSpaces::SpaceId).integer().not_null())
                    .col(ColumnDef::new(GroupSpaces::RoleId).integer().not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_groups_spaces_group_id")
                    .from_tbl(GroupSpaces::Table)
                    .from_col(GroupSpaces::GroupId)
                    .to_tbl(Group::Table)
                    .to_col(Group::GroupId)
                    .to_owned(),
            ).await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_groups_spaces_space_id")
                    .from_tbl(GroupSpaces::Table)
                    .from_col(GroupSpaces::SpaceId)
                    .to_tbl(Space::Table)
                    .to_col(Space::SpaceId)
                    .to_owned(),
            ).await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_groups_spaces_role_id")
                    .from_tbl(GroupSpaces::Table)
                    .from_col(GroupSpaces::RoleId)
                    .to_tbl(Role::Table)
                    .to_col(Role::RoleId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupSpaces::Table).to_owned())
            .await
    }
}

enum GroupSpaces {
    Table,
    Id,
    GroupId,
    SpaceId,
    RoleId
}

// Mapping between Enum variant and its corresponding string value
impl Iden for GroupSpaces {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Groups_Spaces",
                Self::Id => "id",
                Self::GroupId => "group_id",
                Self::SpaceId => "space_id",
                Self::RoleId => "role_id",
            }
        )
            .unwrap();
    }
}