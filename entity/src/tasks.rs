use rocket::{serde::{Deserialize, Serialize}, FromForm};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[field(default = 0)]
    pub id: i32,
    pub item: String,
    #[field(default = 0)]
    pub user_id: i32
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Users
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        // panic!("No RelationDef")
        match self {
            Self::Users => Entity::belongs_to(super::users::Entity)
                .from(Column::UserId)
                .to(super::users::Column::Id)
                .into(),
        }
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}