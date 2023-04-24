use ::entity::{users, users::Entity as User};
use ::entity::{groups, groups::Entity as Group};
use ::entity::{roles, roles::Entity as Role};
use ::entity::{spaces, spaces::Entity as Space};
use ::entity::{posts, posts::Entity as Post};
use ::entity::{comments, comments::Entity as Comment};
use ::entity::{groups_users, groups_users::Entity as UserGroup};
use ::entity::{groups_spaces, groups_spaces::Entity as SpaceGroup};

use sea_orm::*;

pub struct Query;

impl Query {
    /// Users: Find User by ID
    pub async fn find_user_by_id(db: &DbConn, id: i32) -> Result<Option<users::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    /// Users: Find Users in Page
    pub async fn find_users_in_page(
        db: &DbConn,
        page: u64,
        users_per_page: u64,
    ) -> Result<(Vec<users::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = User::find()
            .order_by_asc(users::Column::UserId)
            .paginate(db, users_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Users: Find Users in Name
    pub async fn find_user_by_name(
        db: &DbConn,
        name: String,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find()
            .filter(
                users::Column::UserName
                    .eq(name),
            )
            .one(db)
            .await
    }

    /// Users: Find User in Token
    pub async fn find_user_by_token(
        db: &DbConn,
        token: String,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find()
            .filter(
                users::Column::UserToken
                    .eq(token),
            )
            .one(db)
            .await
    }

    /// User Groups: Get Group Users
    pub async fn find_group_users_in_page(
        db: &DbConn,
        id: i32,
        page: u64,
        users_per_page: u64,
    ) -> Result<(Vec<users::Model>, u64), DbErr> {
        /// Get all group users
        let group_users: Vec<groups_users::Model> = UserGroup::find()
            .filter(groups_users::Column::GroupId.eq(id))
            .all(db)
            .await?;

        /// Get all users
        let paginator = User::find()
            .filter(
                users::Column::UserId
                    .is_in(group_users.iter().map(|u| u.user_id)),
            )
            .order_by_asc(users::Column::UserId)
            .paginate(db, users_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Groups
    pub async fn find_group_by_id(db: &DbConn, id: i32) -> Result<Option<groups::Model>, DbErr> {
        Group::find_by_id(id).one(db).await
    }

    /// Groups: Find Groups in Page
    pub async fn find_groups_in_page(
        db: &DbConn,
        page: u64,
        groups_per_page: u64,
    ) -> Result<(Vec<groups::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Group::find()
            .order_by_asc(groups::Column::GroupId)
            .paginate(db, groups_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated groups
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }


    /// Spaces
    pub async fn find_space_by_id(db: &DbConn, id: i32) -> Result<Option<spaces::Model>, DbErr> {
        Space::find_by_id(id).one(db).await
    }


    /// Spaces: Find Spaces by Owner ID
    pub async fn find_space_by_owner_id(db: &DbConn, id: i32) -> Result<Vec<spaces::Model>, DbErr> {
        Space::find()
            .filter(
                spaces::Column::OwnerId
                    .eq(id),
            )
            .all(db)
            .await
    }

    /// Spaces: Find Posts in Space
    pub async fn find_posts_in_space(
        db: &DbConn,
        space_id: i32,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<posts::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .filter(
                posts::Column::SpaceId
                    .eq(space_id).and(
                    posts::Column::PostPublished
                        .eq(true),
                )
            )
            .order_by_asc(posts::Column::PostId)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Spaces: Find Spaces in Page
    pub async fn find_spaces_in_page(
        db: &DbConn,
        page: u64,
        spaces_per_page: u64,
    ) -> Result<(Vec<spaces::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Space::find()
            .order_by_asc(spaces::Column::SpaceId)
            .paginate(db, spaces_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated spaces
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Spaces: Find Spaces in Page owned by User
    pub async fn find_spaces_in_page_owned_by_user(
        db: &DbConn,
        user_id: i32,
        page: u64,
        spaces_per_page: u64,
    ) -> Result<(Vec<spaces::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Space::find()
            .filter(
                spaces::Column::OwnerId
                    .eq(user_id))
            .order_by_asc(spaces::Column::SpaceId)
            .paginate(db, spaces_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated spaces
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Space Groups: Get Space Groups
    pub async fn find_space_groups_in_page(
        db: &DbConn,
        id: i32,
        page: u64,
        groups_per_page: u64,
    ) -> Result<(Vec<groups::Model>, u64), DbErr> {
        /// Get all space groups
        let group_spaces: Vec<groups_spaces::Model> = SpaceGroup::find()
            .filter(groups_spaces::Column::SpaceId.eq(id))
            .all(db)
            .await?;

        /// Get all groups
        let paginator = Group::find()
            .filter(
                groups::Column::GroupId
                    .is_in(group_spaces.iter().map(|u| u.group_id)),
            )
            .order_by_asc(groups::Column::GroupId)
            .paginate(db, groups_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }


    /// Space Groups: Get Users in Space Groups
    pub async fn find_users_by_space_id(
        db: &DbConn,
        id: i32,
    ) -> Result<Vec<i32>, DbErr> {
        /// Get all space groups
        let group_spaces: Vec<groups_spaces::Model> = SpaceGroup::find()
            .filter(groups_spaces::Column::SpaceId.eq(id))
            .all(db)
            .await?;

        /// Get all groups
        let groups: Vec<groups::Model> = Group::find()
            .filter(
                groups::Column::GroupId
                    .is_in(group_spaces.iter().map(|u| u.group_id)),
            )
            .order_by_asc(groups::Column::GroupId)
            .all(db)
            .await?;

        /// Get all users in groups
        let mut users: Vec<i32> = Vec::new();
        for group in groups {
            let group_users: Vec<groups_users::Model> = UserGroup::find()
                .filter(groups_users::Column::GroupId.eq(group.group_id))
                .all(db)
                .await?;
            users.extend(group_users.iter().map(|u| u.user_id));
        }
        Ok(users)
    }


    /// Posts
    pub async fn find_post_by_id(db: &DbConn, id: i32) -> Result<Option<posts::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    /// Posts: Find Posts in Page
    pub async fn find_posts_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<posts::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .order_by_asc(posts::Column::PostId)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    /// Posts: Find Posts in Page owned by User
    pub async fn find_posts_in_page_owned_by_user(
        db: &DbConn,
        user_id: i32,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<posts::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .filter(
                posts::Column::OwnerId
                    .eq(user_id))
            .order_by_asc(posts::Column::PostId)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
