pub use actix_web;
pub use anyhow;
pub use anyhow::Result;
pub use async_trait::async_trait;
pub use chrono;
pub use chrono::Utc;
pub use colored;
pub use redis;
pub use sea_orm;
pub use tokio;
pub use tracing;
pub use tracing_subscriber;
pub use uuid;

pub mod config;
pub mod db;
pub mod entity;
pub mod handler;
pub mod hyuuid;
pub mod logger;
pub mod permission;
pub mod plugin;
pub mod request;
pub mod utils;
pub use hyuuid::HyUuid;

use chrono::DateTime;
use derivative::Derivative;
use enum_map::EnumMap;
use handler::{GroupHandler, NotificationHandler, PermHandler, SettingHandler, UserHandler};
use logger::Logger;
use parking_lot::RwLock;
use permission::{IDTypes, PermEntry, PermissionItem};
use plugin::PluginManager;
use request::{APIRoute, PaginationParam};
use sea_orm::{
    sea_query::{ConditionExpression, SimpleExpr},
    DatabaseTransaction, EntityTrait, Order, QueryFilter, QueryOrder, Select,
};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::any::Any;
use std::{cmp, collections::HashMap};
use tracing::debug;

/// Make map creation easier.
///
/// # Examples
///
/// ```
/// use skynet::map;
/// let val = map!["key" => "value"];
/// ```
#[macro_export]
macro_rules! map {
    {$($key:expr => $value:expr),+} => {{
        let mut m = std::collections::HashMap::new();
        $(
            m.insert($key, $value);
        )+
        m
    }};
}

/// Get I18n text
///
/// ```ignore
/// // Get a special locale's text
/// t!(skynet, "greeting", "de"); // greeting: "Hallo Welt!" => "Hallo Welt!"
///
/// // With locale and variables
/// t!(skynet, "messages.hello", "de", name = "Jason"); // messages.hello: "Hallo, %{name}" => "Hallo, Jason"
/// ```
#[macro_export]
macro_rules! t {
    ($sk:ident, $key:expr, $locale:expr) => {
        $sk.translate($locale, $key)
    };

    ($sk:ident, $key:expr, $locale:expr, $($var_name:tt = $var_val:expr),+) => {
        {
            let mut message = $sk.translate($locale, $key);
            $(
                message = message.replace(concat!("%{", stringify!($var_name), "}"), $var_val);
            )+
            message
        }
    };
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(i32)]
pub enum NotifyLevel {
    Info = 0,
    Success,
    Warning,
    Error,
}

/// Main entrance providing skynet function.
pub struct Skynet {
    pub user: Box<dyn UserHandler>,
    pub group: Box<dyn GroupHandler>,
    pub perm: Box<dyn PermHandler>,
    pub notification: Box<dyn NotificationHandler>,
    pub setting: Box<dyn SettingHandler>,
    pub logger: Logger,

    pub default_id: EnumMap<IDTypes, HyUuid>,
    pub config: config::Config,
    pub locale: HashMap<String, String>,

    pub plugin: PluginManager,
    pub menu: Vec<MenuItem>,

    pub running: RwLock<bool>,
    pub start_time: DateTime<Utc>,

    pub shared_api: HashMap<HyUuid, Box<dyn Any + Send + Sync>>,
}

impl Drop for Skynet {
    fn drop(&mut self) {
        // clear API first otherwise SIGSEGV.
        self.shared_api.clear();
    }
}

impl Skynet {
    #[allow(clippy::missing_panics_doc)]
    pub fn insert_menu(&mut self, item: MenuItem, pos: usize, parent: Option<HyUuid>) -> bool {
        if let Some(parent) = parent {
            let mut parent: Vec<&mut MenuItem> =
                self.menu.iter_mut().filter(|x| x.id == parent).collect();
            if parent.len() != 1 {
                return false;
            }
            let parent = parent.pop().unwrap();
            let pos = cmp::min(pos, parent.children.len());
            parent.children.insert(pos, item);
        } else {
            let pos = cmp::min(pos, self.menu.len()); // prevent panic
            self.menu.insert(pos, item);
        }
        true
    }

    /// Get merged user permission.
    ///
    /// # Panics
    ///
    /// Panics only when data is corrupted.
    ///
    /// # Errors
    ///
    /// Will return `Err` when db error.
    pub async fn get_user_perm(
        &self,
        db: &DatabaseTransaction,
        uid: &HyUuid,
    ) -> Result<Vec<PermissionItem>> {
        let mut ret: HashMap<String, PermissionItem> = HashMap::new();
        let groups = self.group.find_user_group(db, uid, false).await?;
        for i in groups {
            let perm = self.perm.find_group(db, &i.id).await?;
            for mut j in perm {
                let origin_perm = j.perm;
                if let Some(x) = ret.remove(&j.name) {
                    j.perm |= x.perm;
                    j.created_at = cmp::min(j.created_at, x.created_at);
                    j.updated_at = cmp::max(j.updated_at, x.updated_at);
                    j.origin = x.origin;
                }
                j.origin
                    .push((j.gid.unwrap(), j.ug_name.clone(), origin_perm));
                ret.insert(j.name.clone(), j);
            }
        }
        let users = self.perm.find_user(db, uid).await?;
        for i in users {
            ret.insert(i.name.clone(), i);
        }
        Ok(ret.into_values().collect())
    }

    /// Add new locale items.
    pub fn add_locale(&mut self, l: HashMap<&str, &str>) {
        let count = self.locale.len();
        self.locale
            .extend(l.into_iter().map(|(a, b)| (a.to_owned(), b.to_owned())));
        let new_count = self.locale.len();
        debug!(
            "Locale added: {count} => {new_count} (+{})",
            new_count - count
        );
    }

    /// Translate string.
    /// - Fallback to default language if not exist.
    /// - Again fallback to `key` if still not found.
    pub fn translate(&self, locale: &str, key: &str) -> String {
        let locale_key = format!("{locale}.{key}");
        self.locale.get(locale_key.as_str()).map_or_else(
            || {
                let lang = self.config.lang.get();
                if locale == lang {
                    key.to_owned()
                } else {
                    self.translate(lang, key)
                }
            },
            ToString::to_string,
        )
    }
}

type MenuBadgeFunc = Box<dyn Fn(&Skynet) -> i64 + Send + Sync>;

#[derive(Derivative)]
#[derivative(Default(new = "true"), Debug)]
pub struct MenuItem {
    pub id: HyUuid,
    pub name: String,
    pub path: String,
    pub icon: String,
    pub children: Vec<MenuItem>,
    #[derivative(Debug = "ignore")]
    pub badge_func: Option<MenuBadgeFunc>,
    pub omit_empty: bool,
    pub perm: Option<PermEntry>,
}

impl MenuItem {
    #[must_use]
    pub fn check(&self, p: &HashMap<HyUuid, PermissionItem>) -> bool {
        self.perm.as_ref().map_or(true, |x| x.check(p))
    }
}

pub struct Condition {
    pub page: Option<PaginationParam>,
    pub cond: sea_orm::Condition,
    pub order: Vec<(SimpleExpr, Order)>,
}

impl Condition {
    #[must_use]
    pub fn new(cond: sea_orm::Condition) -> Self {
        Self {
            cond,
            page: None,
            order: Vec::new(),
        }
    }

    #[must_use]
    pub fn default() -> Self {
        Self {
            cond: sea_orm::Condition::all(),
            page: None,
            order: Vec::new(),
        }
    }

    #[must_use]
    pub fn add_sort(mut self, col: SimpleExpr, sort: Order) -> Self {
        self.order.push((col, sort));
        self
    }

    #[must_use]
    pub const fn add_page(mut self, page: PaginationParam) -> Self {
        self.page = Some(page);
        self
    }

    #[must_use]
    pub fn add<C>(mut self, condition: C) -> Self
    where
        C: Into<ConditionExpression>,
    {
        self.cond = self.cond.add(condition);
        self
    }

    #[must_use]
    pub fn add_option<C>(mut self, condition: Option<C>) -> Self
    where
        C: Into<ConditionExpression>,
    {
        self.cond = self.cond.add_option(condition);
        self
    }

    #[must_use]
    pub fn build<E>(self, mut q: Select<E>) -> (Select<E>, Option<PaginationParam>)
    where
        E: EntityTrait,
    {
        for i in self.order {
            q = q.order_by(i.0, i.1);
        }
        (q.filter(self.cond), self.page)
    }
}
