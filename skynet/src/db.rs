#![allow(clippy::enum_glob_use)]
use anyhow::Result;
use enum_map::EnumMap;
use migration::{DbErr, Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, TransactionTrait};

use crate::{
    permission::IDTypes::{self, *},
    HyUuid, Skynet,
};

fn default_perm() -> Vec<(IDTypes, String)> {
    vec![
        (PermManageUserID, "user management".to_owned()),
        (
            PermManageNotificationID,
            "notification management".to_owned(),
        ),
        (PermManageSystemID, "system management".to_owned()),
        (PermManagePluginID, "plugin management".to_owned()),
    ]
}

/// # Errors
///
/// Will return `Err` for db error.
pub async fn connect<S: AsRef<str>>(dsn: S) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(dsn.as_ref().to_owned());
    opt.sqlx_logging(false);
    Database::connect(opt).await
}

/// # Errors
///
/// Will return `Err` for db error.
pub async fn init(db: &DatabaseConnection, skynet: &Skynet) -> Result<EnumMap<IDTypes, HyUuid>> {
    Migrator::up(db, None).await?;

    let mut ret = EnumMap::<IDTypes, HyUuid>::default();
    // default permission
    let tx = db.begin().await?;
    for (id, note) in default_perm() {
        ret[id] = skynet
            .perm
            .find_or_init(&tx, &id.to_string(), &note)
            .await?
            .id;
    }
    tx.commit().await?;
    ret[PermRootID] = HyUuid::nil();
    ret[PermUserID] = HyUuid(uuid::uuid!("1a2d05da-a256-475c-a2b0-dd0aa1b36b4f"));
    ret[PermGuestID] = HyUuid(uuid::uuid!("61ee97f9-0a4b-4215-a9c7-ace22708bb6c"));

    Ok(ret)
}
