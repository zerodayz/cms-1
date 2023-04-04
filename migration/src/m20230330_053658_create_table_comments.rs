use sea_orm_migration::prelude::*;
use crate::m20230330_052948_create_table_users::User;
use crate::m20230330_053719_create_table_posts::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Comment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Comment::CommentId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Comment::CommentContent).string().not_null())
                    .col(ColumnDef::new(Comment::PostId).integer().not_null())
                    .col(ColumnDef::new(Comment::AuthorId).integer().not_null())
                    .col(ColumnDef::new(Comment::CommentCreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
                    .col(ColumnDef::new(Comment::CommentUpdatedAt).timestamp().default(Expr::cust("CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP")).not_null())
                    .to_owned(),
            )
            .await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_comments_post_id")
                    .from_tbl(Comment::Table)
                    .from_col(Comment::PostId)
                    .to_tbl(Post::Table)
                    .to_col(Post::PostId)
                    .to_owned(),
            ).await;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_comments_author_id")
                    .from_tbl(Comment::Table)
                    .from_col(Comment::AuthorId)
                    .to_tbl(User::Table)
                    .to_col(User::UserId)
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comment::Table).to_owned())
            .await
    }
}

enum Comment {
    Table,
    CommentId,
    CommentContent,
    PostId,
    AuthorId,
    CommentCreatedAt,
    CommentUpdatedAt,
}

// Mapping between Enum variant and its corresponding string value
impl Iden for Comment {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "Comments",
                Self::CommentId => "comment_id",
                Self::CommentContent => "comment_content",
                Self::PostId => "post_id",
                Self::AuthorId => "author_id",
                Self::CommentCreatedAt => "created_at",
                Self::CommentUpdatedAt => "updated_at",
            }
        )
            .unwrap();
    }
}