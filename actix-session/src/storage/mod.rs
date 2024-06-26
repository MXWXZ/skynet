//! Pluggable storage backends for session state.

mod interface;
mod session_key;

pub use self::{
    interface::{LoadError, SaveError, SessionStore, UpdateError},
    session_key::SessionKey,
};

#[cfg(feature = "redis-rs-session")]
mod redis_rs;

#[cfg(feature = "redis-rs-session")]
mod utils;

#[cfg(feature = "redis-rs-session")]
pub use redis_rs::{RedisSessionStore, RedisSessionStoreBuilder};
