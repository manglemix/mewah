use std::time::Duration;

use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::value::AnyValue;

#[derive(PartialEq, Eq, Hash, Debug, Deserialize, Serialize, Clone, Copy)]
pub enum LoadDirective {
    /// Loads the asset at startup
    Immediate,
    /// Loads the asset when it is needed
    WhenNeeded,
}

#[derive(PartialEq, Eq, Hash, Debug, Deserialize, Serialize, Clone, Copy)]
pub enum CacheDirective {
    /// Never caches the asset
    DontCache,
    /// Caches the asset until this amount of time has passed since the last access
    Cache(Duration),
    /// Caches the asset until the application terminates
    CacheForever,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct StaticAssetHeader {
    load_directive: LoadDirective,
    cache_directive: CacheDirective,
    index: u64,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize, Serialize, Clone, Copy,
)]
pub struct StaticAssetId(pub(crate) u32);

pub struct DynamicAssets {
    assets: FxHashMap<String, AnyValue>
}