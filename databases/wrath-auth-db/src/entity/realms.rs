//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "realms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    pub realm_type: u8,
    pub flags: u8,
    pub ip: String,
    #[sea_orm(column_type = "Float")]
    pub population: f32,
    pub timezone: u8,
    pub online: u8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::realm_characters::Entity")]
    RealmCharacters,
}

impl Related<super::realm_characters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RealmCharacters.def()
    }
}

impl Related<super::accounts::Entity> for Entity {
    fn to() -> RelationDef {
        super::realm_characters::Relation::Accounts.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::realm_characters::Relation::Realms.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
