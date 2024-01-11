//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    #[sea_orm(unique)]
    pub username: String,
    pub sessionkey: String,
    pub v: String,
    pub s: String,
    pub banned: u8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::account_data::Entity")]
    AccountData,
    #[sea_orm(has_many = "super::realm_characters::Entity")]
    RealmCharacters,
}

impl Related<super::account_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountData.def()
    }
}

impl Related<super::realm_characters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RealmCharacters.def()
    }
}

impl Related<super::realms::Entity> for Entity {
    fn to() -> RelationDef {
        super::realm_characters::Relation::Realms.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::realm_characters::Relation::Accounts.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
