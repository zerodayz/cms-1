use sea_orm_migration::sea_orm::{entity::*, query::*};
use sea_orm_migration::prelude::*;

use crate::m20230330_053015_create_table_roles::Role;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table(Role::Table)
            .columns([Role::RoleName, Role::RoleDescription])
            .values_panic(["admin".into(), "Admin Role".into()])
            .to_owned();

        manager.exec_stmt(insert).await?;

        let insert = Query::insert()
            .into_table(Role::Table)
            .columns([Role::RoleName, Role::RoleDescription])
            .values_panic(["viewer".into(), "Viewer Role".into()])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete = Query::delete()
            .from_table(Role::Table)
            .and_where(Expr::col(Role::RoleName).eq("admin"))
            .to_owned();

        manager.exec_stmt(delete).await?;

        let delete = Query::delete()
            .from_table(Role::Table)
            .and_where(Expr::col(Role::RoleName).eq("viewer"))
            .to_owned();

        manager.exec_stmt(delete).await?;

        Ok(())
    }
}