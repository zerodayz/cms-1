pub use sea_orm_migration::prelude::*;

mod m20230330_052948_create_table_users;
mod m20230330_053012_create_table_groups;
mod m20230330_053015_create_table_roles;
mod m20230330_053017_create_table_spaces;
mod m20230330_053039_create_table_groups_users;
mod m20230330_053048_create_table_groups_spaces;
mod m20230330_053658_create_table_comments;
mod m20230330_053719_create_table_posts;
mod m20230412_044401_create_default_roles;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230330_052948_create_table_users::Migration),
            Box::new(m20230330_053012_create_table_groups::Migration),
            Box::new(m20230330_053015_create_table_roles::Migration),
            Box::new(m20230330_053017_create_table_spaces::Migration),
            Box::new(m20230330_053039_create_table_groups_users::Migration),
            Box::new(m20230330_053048_create_table_groups_spaces::Migration),
            Box::new(m20230330_053658_create_table_comments::Migration),
            Box::new(m20230330_053719_create_table_posts::Migration),
            Box::new(m20230412_044401_create_default_roles::Migration),
        ]
    }
}
