use sea_orm_migration::prelude::*;
use crate::m20230330_052948_create_table_users::User;
use crate::m20230330_053017_create_table_spaces::Space;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Post::PostId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Post::PostTitle).string().not_null())
                    .col(ColumnDef::new(Post::PostContent).text().not_null())
                    .col(ColumnDef::new(Post::PostPublished).boolean().not_null().default(false))
                    .col(ColumnDef::new(Post::SpaceId).integer().not_null())
                    .col(ColumnDef::new(Post::OwnerId).integer().not_null())
                    .col(ColumnDef::new(Post::PostCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(Post::PostUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_posts_space_id")
                    .from_tbl(Post::Table)
                    .from_col(Post::SpaceId)
                    .to_tbl(Space::Table)
                    .to_col(Space::SpaceId)
                    .to_owned(),
            ).await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_posts_owner_id")
                    .from_tbl(Post::Table)
                    .from_col(Post::OwnerId)
                    .to_tbl(User::Table)
                    .to_col(User::UserId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}

pub enum Post {
    Table,
    PostId,
    PostTitle,
    PostContent,
    PostPublished,
    SpaceId,
    OwnerId,
    PostCreatedAt,
    PostUpdatedAt,
}

// Mapping between Enum variant and its corresponding string value
impl Iden for Post {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Posts",
                Self::PostId => "post_id",
                Self::PostTitle => "post_title",
                Self::PostContent => "post_content",
                Self::PostPublished => "post_published",
                Self::SpaceId => "space_id",
                Self::OwnerId => "owner_id",
                Self::PostCreatedAt => "created_at",
                Self::PostUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}