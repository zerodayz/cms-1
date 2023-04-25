use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend, ExecResult, Statement};
use crate::m20230330_053719_create_table_posts::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Update posts PostContent column from text to mediumtext
        // There is no way to do it with modify_column
        let db = manager.get_connection();
        let exec_res: ExecResult = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                "ALTER TABLE `Posts` MODIFY COLUMN `post_content` MEDIUMTEXT NOT NULL;".to_owned(),
            ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Update posts PostContent column from mediumtext to text
        // There is no way to do it with modify_column
        let db = manager.get_connection();
        let exec_res: ExecResult = db
            .execute(Statement::from_string(
                DatabaseBackend::MySql,
                "ALTER TABLE `Posts` MODIFY COLUMN `post_content` TEXT NOT NULL;".to_owned(),
            ))
            .await?;
        Ok(())
    }
}