use derivative::Derivative;
use entity::agents;
use enum_as_inner::EnumAsInner;
use parking_lot::RwLock;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use skynet::{
    anyhow::{self, Result},
    sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set,
        Unchanged,
    },
    uuid::uuid,
    HyUuid, Skynet,
};
use std::{collections::HashMap, time};

pub mod client;
pub mod entity;
pub mod server;

pub static ID: HyUuid = HyUuid(uuid!("2eb2e1a5-66b4-45f9-ad24-3c4f05c858aa"));

#[derive(Derivative)]
#[derivative(Default(new = "true"))]
pub struct PluginSrv {
    pub view_id: HyUuid,
    pub manage_id: HyUuid,
    pub agent: RwLock<HashMap<HyUuid, Agent>>,
}

#[derive(
    Default, EnumAsInner, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Hash, Clone, Copy,
)]
#[repr(u8)]
pub enum AgentStatus {
    #[default]
    Offline = 0,
    Online,
    Updating,
}

#[derive(Derivative, Serialize)]
#[derivative(Default(new = "true"))]
pub struct Agent {
    pub id: HyUuid,
    pub uid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    pub last_login: i64,
    pub status: AgentStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rsp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>, // cpu status, unit percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<i64>, // memory status, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory: Option<i64>, // total memory, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<i64>, // disk status, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_disk: Option<i64>, // total disk, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<i64>, // agent latency, unit ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_up: Option<i64>, // network upload, unit bytes/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_down: Option<i64>, // network download, unit bytes/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_up: Option<i64>, // bandwidth upload, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_down: Option<i64>, // bandwidth download, unit bytes
}

impl From<agents::Model> for Agent {
    fn from(v: agents::Model) -> Self {
        Self {
            id: v.id,
            uid: v.uid,
            name: v.name,
            os: v.os,
            hostname: v.hostname,
            ip: v.ip,
            system: v.system,
            arch: v.arch,
            last_login: v.last_login,
            ..Default::default()
        }
    }
}

impl PluginSrv {
    #[must_use]
    pub fn setting_name() -> String {
        format!("plugin_{ID}_token")
    }

    pub fn get_setting(skynet: &Skynet) -> Option<String> {
        skynet.setting.get(&Self::setting_name())
    }

    /// # Errors
    ///
    /// Will raise `Err` for db errors.
    pub async fn set_setting(db: &DatabaseTransaction, skynet: &Skynet, token: &str) -> Result<()> {
        skynet.setting.set(db, &Self::setting_name(), token).await
    }

    #[allow(clippy::needless_collect)]
    pub async fn init(&self, db: &DatabaseTransaction) -> Result<()> {
        let agents: Vec<Agent> = agents::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(From::from)
            .collect();
        let mut wlock = self.agent.write();
        for x in agents {
            wlock.insert(x.id, x);
        }
        Ok(())
    }

    pub async fn update(
        &self,
        db: &DatabaseTransaction,
        id: &HyUuid,
        os: Option<String>,
        system: Option<String>,
        arch: Option<String>,
        hostname: Option<String>,
    ) -> Result<()> {
        agents::ActiveModel {
            id: Unchanged(*id),
            os: Set(os.clone()),
            system: Set(system.clone()),
            arch: Set(arch.clone()),
            hostname: Set(hostname.clone()),
            ..Default::default()
        }
        .update(db)
        .await?;
        let mut wlock = self.agent.write();
        if let Some(item) = wlock.get_mut(id) {
            item.os = os;
            item.system = system;
            item.arch = arch;
            item.hostname = hostname;
        }
        Ok(())
    }

    pub async fn delete(&self, db: &DatabaseTransaction, id: &HyUuid) -> Result<u64> {
        let num = agents::Entity::delete_by_id(*id)
            .exec(db)
            .await?
            .rows_affected;
        self.agent.write().remove(id);
        Ok(num)
    }

    pub async fn rename(&self, db: &DatabaseTransaction, id: &HyUuid, name: &str) -> Result<()> {
        agents::ActiveModel {
            id: Unchanged(*id),
            name: Set(name.to_owned()),
            ..Default::default()
        }
        .update(db)
        .await?;
        let mut wlock = self.agent.write();
        if let Some(item) = wlock.get_mut(id) {
            item.name = name.to_owned();
        }
        Ok(())
    }

    pub async fn find_by_id(
        &self,
        db: &DatabaseTransaction,
        id: &HyUuid,
    ) -> Result<Option<agents::Model>> {
        agents::Entity::find_by_id(*id)
            .one(db)
            .await
            .map_err(anyhow::Error::from)
    }

    pub async fn find_by_uid(
        &self,
        db: &DatabaseTransaction,
        uid: &str,
    ) -> Result<Option<agents::Model>> {
        agents::Entity::find()
            .filter(agents::Column::Uid.eq(uid))
            .one(db)
            .await
            .map_err(anyhow::Error::from)
    }

    pub async fn find_by_name(
        &self,
        db: &DatabaseTransaction,
        name: &str,
    ) -> Result<Option<agents::Model>> {
        agents::Entity::find()
            .filter(agents::Column::Name.eq(name))
            .one(db)
            .await
            .map_err(anyhow::Error::from)
    }

    pub fn update_state(&self, id: &HyUuid) {
        let mut wlock = self.agent.write();
        if let Some(item) = wlock.get_mut(id) {
            item.status = AgentStatus::Updating;
        }
    }

    pub fn logout(&self, id: &HyUuid) {
        let mut wlock = self.agent.write();
        if let Some(item) = wlock.get_mut(id) {
            item.status = AgentStatus::Offline;
        }
    }

    pub async fn login(
        &self,
        db: &DatabaseTransaction,
        uid: String,
        ip: String,
    ) -> Result<Option<HyUuid>> {
        let agent = self.find_by_uid(db, &uid).await?;
        let now: i64 = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
        let agent = if let Some(agent) = agent {
            agent
        } else {
            agents::ActiveModel {
                uid: Set(uid.clone()),
                name: Set(uid.chars().take(8).collect()),
                ip: Set(ip.clone()),
                last_login: Set(now),
                ..Default::default()
            }
            .insert(db)
            .await?
        };
        let status = self.agent.read().get(&agent.id).map(|x| x.status);
        if let Some(status) = status {
            if status.is_offline() {
                let mut agent: agents::ActiveModel = agent.into();
                agent.ip = Set(ip.clone());
                agent.last_login = Set(now);
                let agent = agent.update(db).await?;

                let mut wlock = self.agent.write();
                let item = wlock.get_mut(&agent.id).unwrap();
                item.ip = ip;
                item.last_login = now;
                item.status = AgentStatus::Online;
                Ok(Some(agent.id))
            } else {
                Ok(None)
            }
        } else {
            let mut agent: Agent = agent.into();
            agent.status = AgentStatus::Online;
            let id = agent.id;
            self.agent.write().insert(id, agent);
            Ok(Some(id))
        }
    }
}
