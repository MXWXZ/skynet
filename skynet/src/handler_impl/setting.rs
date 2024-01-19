use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use derivative::Derivative;
use migration::Expr;
use parking_lot::RwLock;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set, Value,
};
use skynet::{entity::settings, handler::SettingHandler};

#[derive(Derivative)]
#[derivative(Default(new = "true"), Debug)]
pub struct DefaultSettingHandler {
    cache: RwLock<HashMap<String, String>>,
}

#[async_trait]
impl SettingHandler for DefaultSettingHandler {
    async fn build_cache(&self, db: &DatabaseTransaction) -> Result<()> {
        let res = settings::Entity::find().all(db).await?;
        let mut cache = self.cache.write();
        for i in res {
            cache.insert(i.name, i.value);
        }
        drop(cache);
        Ok(())
    }

    fn get_all(&self) -> HashMap<String, String> {
        self.cache.read().clone()
    }

    fn get(&self, name: &str) -> Option<String> {
        self.cache.read().get(name).cloned()
    }

    async fn set(&self, db: &DatabaseTransaction, name: &str, value: &str) -> Result<()> {
        let v = self.cache.read().get(name).cloned();
        if let Some(x) = v {
            if x != value {
                settings::Entity::update_many()
                    .filter(settings::Column::Name.eq(name))
                    .col_expr(
                        settings::Column::Value,
                        Expr::value(Value::String(Some(Box::new(value.to_owned())))),
                    )
                    .exec(db)
                    .await?;
                self.cache.write().insert(name.to_owned(), value.to_owned());
            }
        } else {
            settings::ActiveModel {
                name: Set(name.to_owned()),
                value: Set(value.to_owned()),
                ..Default::default()
            }
            .insert(db)
            .await?;
            self.cache.write().insert(name.to_owned(), value.to_owned());
        }
        Ok(())
    }

    async fn delete(&self, db: &DatabaseTransaction, name: &str) -> Result<bool> {
        if self.cache.read().contains_key(name) {
            let rows = settings::Entity::delete_many()
                .filter(settings::Column::Name.eq(name))
                .exec(db)
                .await
                .map(|x| x.rows_affected)?;
            self.cache.write().remove(name);
            Ok(rows == 1)
        } else {
            Ok(false)
        }
    }

    async fn delete_all(&self, db: &DatabaseTransaction) -> Result<u64> {
        let rows = settings::Entity::delete_many()
            .exec(db)
            .await
            .map(|x| x.rows_affected)?;
        self.cache.write().clear();
        Ok(rows)
    }
}
