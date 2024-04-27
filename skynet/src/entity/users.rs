//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;
use serde::Serialize;
use skynet_macro::{entity_behavior, entity_id, entity_timestamp};

use crate::{utils::vec_string_option, HyUuid};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: HyUuid,
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(serialize_with = "vec_string_option")]
    pub avatar: Option<Vec<u8>>,
    pub last_login: Option<i64>,
    pub last_ip: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_group_links::Relation::Group.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::user_group_links::Relation::User.def().rev())
    }
}

impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
        super::permission_links::Relation::Permission.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::permission_links::Relation::User.def().rev())
    }
}

#[entity_id]
#[entity_timestamp]
impl ActiveModel {}

#[entity_behavior]
impl ActiveModelBehavior for ActiveModel {}
