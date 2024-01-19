use std::{
    cmp,
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    ops::Index,
    path::{Path, PathBuf},
    result,
    sync::Arc,
};

use anyhow::{bail, Result};
use derivative::Derivative;
use enum_as_inner::EnumAsInner;
use libloading::{Library, Symbol};
use log::{debug, error};
use parking_lot::RwLock;
use rs_config::Config;
use sea_orm::DatabaseTransaction;
use semver::Version;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, DisplayFromStr};
use walkdir::WalkDir;

use crate::{logger::Logger, APIRoute, HyUuid, Skynet};

const PLUGIN_SETTING_PREFIX: &str = "plugin_";
const PLUGIN_CREATE: &[u8] = b"_plugin_create";
const PLUGIN_CONFIG: &str = "config.yml";

/// Plugin interface, all plugins should implement this trait.
///
/// # Lifecycle
///
/// - Skynet init(db, redis, etc.)
/// - Check plugin enabled
/// - **<`_init_logger`>**
/// - **<`on_load`>**
/// - **<`on_route`>**
/// - Skynet running
/// - ...
/// - **<`on_unload`>**
/// - Skynet shutdown
pub trait Plugin: Send + Sync {
    /// Fired to init logger.
    ///
    /// # Warning
    ///
    /// Do not change this.
    ///
    /// # Errors
    ///
    /// Will return `Err` if logger cannot be set.
    fn _init_logger(&self, s: &mut Logger) -> Result<()> {
        s.reinit()
    }

    /// Fired when the plugin is loaded.
    fn on_load(&self, s: Skynet) -> (Skynet, Result<()>) {
        (s, Ok(()))
    }

    /// Fired when applying routes.
    fn on_route(&self, r: Vec<APIRoute>) -> Vec<APIRoute> {
        r
    }

    /// Fired when the plugin is unloaded.
    fn on_unload(&self, _enable: PluginStatus) {}
}

/// Create a plugin.
///
/// # Example
///
/// ```
/// #[derive(Debug, Default)]
/// struct YourPlugin;
///
/// impl Plugin for YourPlugin {
/// // your implementation
/// }
///
/// create_plugin!(YourPlugin, YourPlugin::default);
/// ```
#[macro_export]
macro_rules! create_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::plugin::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let boxed: Box<dyn $crate::plugin::Plugin> = Box::new(constructor());
            Box::into_raw(boxed)
        }
    };
}

#[derive(thiserror::Error, Derivative)]
#[derivative(Debug)]
pub enum PluginError {
    #[error("Cannot parse plugin config path")]
    ConfigPath(OsString),

    #[error("Plugin `{1}` and `{2}` have conflict id `{0}`")]
    ConflictID(HyUuid, String, String),
}

#[derive(
    Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumAsInner,
)]
#[repr(u8)]
pub enum PluginStatus {
    Unload = 0,
    PendingDisable,
    PendingEnable,
    Enable,
}

#[serde_as]
#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct PluginInstance {
    pub id: HyUuid,
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub version: Version,
    pub priority: i32,
    pub status: PluginStatus,
    pub path: String,

    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    instance: Option<Box<dyn Plugin>>,
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    library: Option<Library>,
}

impl PluginInstance {
    pub fn setting_name(&self) -> String {
        format!("{}{}", PLUGIN_SETTING_PREFIX, self.id)
    }

    pub async fn set(
        &mut self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        enable: bool,
    ) -> Result<()> {
        match self.status {
            PluginStatus::Unload => {
                if enable {
                    skynet
                        .setting
                        .set(db, &self.setting_name(), "1")
                        .await
                        .and_then(|_| Ok(self.status = PluginStatus::PendingEnable))
                } else {
                    Ok(())
                }
            }
            PluginStatus::PendingDisable => {
                if enable {
                    skynet
                        .setting
                        .set(db, &self.setting_name(), "1")
                        .await
                        .and_then(|_| Ok(self.status = PluginStatus::Enable))
                } else {
                    Ok(())
                }
            }
            PluginStatus::PendingEnable => {
                if enable {
                    Ok(())
                } else {
                    skynet
                        .setting
                        .set(db, &self.setting_name(), "0")
                        .await
                        .and_then(|_| Ok(self.status = PluginStatus::Unload))
                }
            }
            PluginStatus::Enable => {
                if enable {
                    Ok(())
                } else {
                    skynet
                        .setting
                        .set(db, &self.setting_name(), "0")
                        .await
                        .and_then(|_| Ok(self.status = PluginStatus::PendingDisable))
                }
            }
        }
    }
}

impl PartialOrd for PluginInstance {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PluginInstance {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialEq for PluginInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PluginInstance {}

#[derive(Derivative)]
#[derivative(Default(new = "true"), Debug)]
pub struct PluginManager {
    pub plugin: RwLock<Vec<Arc<PluginInstance>>>,
}

impl PluginManager {
    pub fn get(&self, id: &HyUuid) -> Option<Arc<PluginInstance>> {
        for i in self.plugin.read().iter() {
            if i.id == *id {
                return Some(i.clone());
            }
        }
        None
    }

    pub async fn set(
        &self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        id: &HyUuid,
        enable: bool,
    ) -> Result<bool> {
        for i in self.plugin.write().iter_mut() {
            if let Some(x) = Arc::get_mut(i) {
                if x.id == *id {
                    return x.set(db, skynet, enable).await.and(Ok(true));
                }
            }
        }
        Ok(false)
    }

    pub async fn load<P: AsRef<Path>>(
        &self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        path: P,
    ) -> Result<bool> {
        let mut dll: Vec<PathBuf> = WalkDir::new(path.as_ref())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|x| {
                x.path()
                    .extension()
                    .is_some_and(|x| x == env::consts::DLL_EXTENSION)
            })
            .map(|x| x.map_or(PathBuf::new(), |x| x.path().to_path_buf()))
            .collect();
        if dll.len() != 1 {
            return Ok(false);
        }
        let dll = dll.pop().unwrap();
        let inst = self.load_internal(path.as_ref().join(PLUGIN_CONFIG), dll)?;
        if let Some(x) = self.get(&inst.id) {
            bail!(PluginError::ConflictID(x.id, inst.name, x.name.clone()));
        }
        skynet.setting.set(db, &inst.setting_name(), "0").await?;
        let mut wlock = self.plugin.write();
        for i in 1..wlock.len() {
            if wlock.index(i).priority > inst.priority {
                wlock.insert(i, Arc::new(inst));
                return Ok(true);
            }
        }
        wlock.push(Arc::new(inst));
        Ok(true)
    }

    /// Load all plugins in folder, ignore error.
    ///
    /// # Panics
    ///
    /// Panics if `dir` cannot be parsed or db error.
    pub fn load_all<P: AsRef<Path>>(&mut self, skynet: Skynet, dir: P) -> Skynet {
        let mut instance = Vec::new();
        let mut conflict_id: HashMap<HyUuid, String> = HashMap::new();
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_entry(|x| {
                x.path()
                    .extension()
                    .is_some_and(|x| x == env::consts::DLL_EXTENSION)
            })
            .filter_map(result::Result::ok)
        {
            match self.load_internal(
                entry.path().parent().unwrap().join(PLUGIN_CONFIG),
                entry.path().into(),
            ) {
                Ok(obj) => {
                    if let Some(x) = conflict_id.get(&obj.id) {
                        panic!(
                            "{}",
                            PluginError::ConflictID(obj.id, obj.name, x.to_owned())
                        );
                    }
                    conflict_id.insert(obj.id, obj.name.clone());
                    instance.push(obj)
                }
                Err(e) => error!("Plugin `{:?}` read error: `{}`", entry.path(), e),
            }
        }
        instance.sort();

        for i in &mut instance {
            if skynet
                .setting
                .get(&i.setting_name())
                .is_some_and(|x| x == "1")
            {
                i.status = PluginStatus::Enable;
            }
        }

        let mut skynet = skynet;
        for i in &mut instance {
            if i.status.is_enable() {
                let inst = i.instance.as_ref().unwrap();
                if let Err(e) = inst._init_logger(&mut skynet.logger) {
                    i.status = PluginStatus::Unload;
                    error!(
                        "Plugin {}({}) unload because of `_init_logger` error: {}",
                        i.id, i.name, e,
                    );
                } else {
                    let load_res = inst.on_load(skynet);
                    skynet = load_res.0;
                    match load_res.1 {
                        Ok(()) => debug!("Plugin loaded: {}({})", i.id, i.name),
                        Err(e) => {
                            i.status = PluginStatus::Unload;
                            error!(
                                "Plugin {}({}) unload because of `on_load` error: {}",
                                i.id, i.name, e,
                            );
                        }
                    }
                }
            }
            if !i.status.is_enable() {
                i.library = None;
                i.instance = None;
            }
        }
        self.plugin = RwLock::new(instance.into_iter().map(|x| Arc::new(x)).collect());
        skynet
    }

    /// Load plugin .dll/.so/.dylib file.
    fn load_internal<P: AsRef<OsStr>>(&self, config: P, filename: P) -> Result<PluginInstance> {
        let config = config
            .as_ref()
            .to_str()
            .ok_or_else(|| PluginError::ConfigPath(config.as_ref().to_os_string()))?;
        let settings = Config::builder()
            .add_source(rs_config::File::with_name(config))
            .build()?;

        // SAFETY: plugin load must be unsafe.
        unsafe {
            type PluginCreate = unsafe fn() -> *mut dyn Plugin;
            let lib = Library::new(filename.as_ref())?;
            let constructor: Symbol<PluginCreate> = lib.get(PLUGIN_CREATE)?;
            let plugin = Box::from_raw(constructor());
            let mut path = PathBuf::from(filename.as_ref()).canonicalize()?;
            path.pop();

            Ok(PluginInstance {
                id: HyUuid::parse(&settings.get_string("id")?)?,
                name: settings.get_string("name")?,
                version: Version::parse(&settings.get_string("version")?)?,
                priority: settings.get_int("priority")?.try_into()?,
                path: path.file_name().unwrap().to_string_lossy().to_string(),
                status: PluginStatus::Unload,
                instance: Some(plugin),
                library: Some(lib),
            })
        }
    }

    /// Parse route.
    #[must_use]
    pub fn parse_route(&self, route: Vec<APIRoute>) -> Vec<APIRoute> {
        let mut route = route;
        for i in self.plugin.read().iter() {
            if let Some(x) = &i.instance {
                route = x.on_route(route);
            }
        }
        route
    }

    /// Unload all plugins.
    fn unloadall(&mut self) {
        for plugin in self.plugin.write().drain(..) {
            if let Some(x) = &plugin.instance {
                x.on_unload(plugin.status);
                debug!("Plugin unloaded: {}({})", plugin.id, plugin.name);
            }
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.unloadall();
    }
}