use sea_orm::{entity::prelude::*, Set};
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[serde(rename_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "type")]
pub enum AppType {
    #[sea_orm(string_value = "docker")]
    Docker,
    #[sea_orm(string_value = "static")]
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "apps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, unique)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub name: String,

    #[serde(rename = "type")]
    #[sea_orm(column_name = "type")]
    pub type_: AppType,
}

#[derive(Debug, Copy, Clone, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }
}

pub fn all_docker_apps() -> Select<Entity> {
    Entity::find().filter(Column::Type.eq(AppType::Docker))
}

pub fn all_static_apps() -> Select<Entity> {
    Entity::find().filter(Column::Type.eq(AppType::Static))
}
