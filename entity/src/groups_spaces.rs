//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "Groups_Spaces")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub group_id: i32,
    pub space_id: i32,
    pub role_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberForm {
    pub group_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MembersForm {
    pub group_ids: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupId",
        to = "super::groups::Column::GroupId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Groups,
    #[sea_orm(
        belongs_to = "super::spaces::Entity",
        from = "Column::SpaceId",
        to = "super::spaces::Column::SpaceId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Spaces,
    #[sea_orm(
    belongs_to = "super::roles::Entity",
    from = "Column::RoleId",
    to = "super::roles::Column::RoleId",
    on_update = "Restrict",
    on_delete = "Restrict"
    )]
    Roles,
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groups.def()
    }
}

impl Related<super::spaces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Spaces.def()
    }
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roles.def()
    }
}


impl ActiveModelBehavior for ActiveModel {}
