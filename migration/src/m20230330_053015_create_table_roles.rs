use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Role::RoleId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Role::RoleName).string().not_null())
                    .col(ColumnDef::new(Role::RoleDescription).string().not_null())
                    .col(ColumnDef::new(Role::RoleCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(Role::RoleUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await
    }
}

pub enum Role {
    Table,
    RoleId,
    RoleName,
    RoleDescription,
    RoleCreatedAt,
    RoleUpdatedAt,
}

// Mapping between Enum variant and its corresponding string value
impl Iden for Role {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Roles",
                Self::RoleId => "role_id",
                Self::RoleName => "role_name",
                Self::RoleDescription => "role_description",
                Self::RoleCreatedAt => "created_at",
                Self::RoleUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}