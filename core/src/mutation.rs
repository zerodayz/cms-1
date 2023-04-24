use ::entity::{users, users::Entity as User};
use ::entity::{groups, groups::Entity as Group};
use ::entity::{roles, roles::Entity as Role};
use ::entity::{spaces, spaces::Entity as Space};
use ::entity::{posts, posts::Entity as Post};
use ::entity::{comments, comments::Entity as Comment};
use ::entity::{groups_users, groups_users::Entity as UserGroup};
use ::entity::{groups_spaces, groups_spaces::Entity as SpaceGroup};

use sea_orm::*;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

pub struct Mutation;

impl Mutation {

    fn hash_password(password: String) -> String {
        let salt = bcrypt::DEFAULT_COST;
        bcrypt::hash(password, salt).unwrap()
    }

    pub fn verify_password(password: String, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap()
    }

    /// Users: Create User
    pub async fn create_user(
        db: &DbConn,
        user: users::Model,
    ) -> Result<users::ActiveModel, DbErr> {
        users::ActiveModel {
            user_name: Set(user.user_name.to_owned()),
            user_password: Set(Self::hash_password(user.user_password.to_owned())),
            user_token: Set("ThisIsMySecretToken".to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }

    /// Users: Delete User
    pub async fn delete_user(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr>  {
        let user: users::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.delete(db).await
    }
    /// Users: Update User
    pub async fn update_user_by_id(
        db: &DbConn,
        id: i32,
        form_data: users::Model,
    ) -> Result<users::Model, DbErr> {
        let user: users::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        users::ActiveModel {
            user_id: user.user_id,
            user_name: Set(form_data.user_name.to_owned()),
            user_password: Set(Self::hash_password(form_data.user_password.to_owned())),
            user_token: user.user_token,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
        .update(db)
        .await
    }

    /// Users: Update User Token
    pub async fn update_user_token(
        db: &DbConn,
        id: i32,
        token: &String,
    ) -> Result<users::Model, DbErr> {
        let user: users::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        users::ActiveModel {
            user_id: user.user_id,
            user_name: user.user_name,
            user_password: user.user_password,
            user_token: Set(token.to_string()),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
        .update(db)
        .await
    }

    /// User Groups: Remove Users from Group
    pub async fn remove_users_from_group(
        db: &DbConn,
        group_id: i32,
        user_ids: Vec<i32>,
    ) -> Result<DeleteResult, DbErr> {
        let res: DeleteResult = UserGroup::delete_many()
            .filter(
                groups_users::Column::GroupId.eq(group_id)
                    .and(groups_users::Column::UserId.is_in(user_ids)),
            )
            .exec(db)
            .await?;
        Ok(res)
    }

    /// User Groups: Add Users from Group
    pub async fn add_users_into_group(
        db: &DbConn,
        group_id: i32,
        user_ids: Vec<i32>,
    ) -> Result<(), DbErr> {
        for user_id in user_ids {
            let user = groups_users::ActiveModel {
                group_id: Set(group_id),
                user_id: Set(user_id),
                ..Default::default()
            };
            user.insert(db).await?;
        }
        Ok(())
    }

    /// Spaces: Create Space
    pub async fn create_space(
        db: &DbConn,
        logged_user: i32,
        form_data: spaces::Model,
    ) -> Result<spaces::ActiveModel, DbErr> {
        spaces::ActiveModel {
            space_name: Set(form_data.space_name.to_owned()),
            owner_id: Set(Option::from(logged_user)),
            is_public: Set(form_data.is_public),
            ..Default::default()
        }
            .save(db)
            .await
    }
    /// Spaces: Delete Space
    pub async fn delete_space(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr>  {
        let space: spaces::ActiveModel = Space::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find space.".to_owned()))
            .map(Into::into)?;

        space.delete(db).await
    }
    /// Spaces: Update Space
    pub async fn update_space_by_id(
        db: &DbConn,
        id: i32,
        form_data: spaces::Model,
    ) -> Result<spaces::Model, DbErr> {
        let space: spaces::ActiveModel = Space::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find space.".to_owned()))
            .map(Into::into)?;

        spaces::ActiveModel {
            space_id: space.space_id,
            space_name: Set(form_data.space_name.to_owned()),
            is_public: Set(form_data.is_public),
            owner_id: space.owner_id,
            created_at: space.created_at,
            updated_at: space.updated_at,
        }
            .update(db)
            .await
    }


    /// Space Groups: Remove Groups from Space
    pub async fn remove_groups_from_space(
        db: &DbConn,
        space_id: i32,
        group_ids: Vec<i32>,
    ) -> Result<DeleteResult, DbErr> {
        let res: DeleteResult = SpaceGroup::delete_many()
            .filter(
                groups_spaces::Column::SpaceId.eq(space_id)
                    .and(groups_spaces::Column::GroupId.is_in(group_ids)),
            )
            .exec(db)
            .await?;
        Ok(res)
    }

    /// Space Groups: Add Group into Space
    pub async fn add_groups_into_space(
        db: &DbConn,
        space_id: i32,
        group_ids: Vec<i32>,
    ) -> Result<(), DbErr> {
        let role: roles::ActiveModel = Role::find()
            .filter(roles::Column::RoleName.eq("viewer"))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find role.".to_owned()))
            .map(Into::into)?;

        for group_id in group_ids {
            let group_spaces = groups_spaces::ActiveModel {
                group_id: Set(group_id),
                space_id: Set(space_id),
                role_id: Set(role.role_id.clone().unwrap()),
                ..Default::default()
            };
            group_spaces.insert(db).await?;
        }
        Ok(())
    }

    /// Groups: Create Group
    pub async fn create_group(
        db: &DbConn,
        form_data: groups::Model,
    ) -> Result<groups::ActiveModel, DbErr> {
        groups::ActiveModel {
            group_name: Set(form_data.group_name.to_owned()),
            owner_id: Set(form_data.owner_id.to_owned()),
            ..Default::default()
        }
            .save(db)
            .await
    }
    /// Groups: Delete Group
    pub async fn delete_group(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr>  {
        let group: groups::ActiveModel = Group::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find group.".to_owned()))
            .map(Into::into)?;

        group.delete(db).await
    }
    /// Groups: Update Group
    pub async fn update_group_by_id(
        db: &DbConn,
        id: i32,
        form_data: groups::Model,
    ) -> Result<groups::Model, DbErr> {
        let group: groups::ActiveModel = Group::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find group.".to_owned()))
            .map(Into::into)?;

        groups::ActiveModel {
            group_id: group.group_id,
            group_name: Set(form_data.group_name.to_owned()),
            owner_id: Set(form_data.owner_id.to_owned()),
            created_at: group.created_at,
            updated_at: group.updated_at,
        }
            .update(db)
            .await
    }
    
    /// Posts: Create Post
    pub async fn create_post(
        db: &DbConn,
        logged_user: i32,
        form_data: posts::Model,
    ) -> Result<posts::ActiveModel, DbErr> {
        let title = form_data.post_content.split("<h1>").nth(1).unwrap().split("</h1>").nth(0).unwrap();
        let content = form_data.post_content.split("<h1>").nth(1).unwrap().split("</h1>").nth(1).unwrap();

        posts::ActiveModel {
            post_title: Set(title.to_owned()),
            post_content: Set(content.to_owned()),
            post_published: Set(form_data.post_published),
            space_id: Set(form_data.space_id),
            owner_id: Set(logged_user),
            ..Default::default()
        }
        .save(db)
        .await
    }
    /// Posts: Update Post by ID
    pub async fn update_post_by_id(
        db: &DbConn,
        id: i32,
        form_data: posts::Model,
    ) -> Result<posts::Model, DbErr> {
        let post: posts::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        let title = form_data.post_content.split("<h1>").nth(1).unwrap().split("</h1>").nth(0).unwrap();
        let content = form_data.post_content.split("<h1>").nth(1).unwrap().split("</h1>").nth(1).unwrap();

        posts::ActiveModel {
            post_id: post.post_id,
            post_title: Set(title.to_owned()),
            post_content: Set(content.to_owned()),
            post_published: Set(form_data.post_published),
            space_id: Set(form_data.space_id),
            owner_id: post.owner_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
        .update(db)
        .await
    }
    /// Posts: Delete Post
    pub async fn delete_post(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let post: posts::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }
    /// Posts: Delete all Posts
    pub async fn delete_all_posts(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Post::delete_many().exec(db).await
    }
}
