#![feature(
    alloc_layout_extra,
    ptr_internals,
    slice_ptr_len,
    slice_ptr_get,
    int_roundings
)]

use std::
    io::{BufReader, Read};

use assets::{StaticAssetId, StaticAssetHeader};
use component::ComponentType;
use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

mod component;
mod assets;
mod value;

#[derive(Deserialize, Serialize)]
pub struct CompiledApplicationHeader {
    static_asset_headers: FxHashMap<StaticAssetId, StaticAssetHeader>,
    components: Vec<ComponentType>,
}

pub fn run_compiled_application(mut reader: BufReader<&mut impl Read>) -> anyhow::Result<()> {
    let mut header_size = [0u8; 8];
    reader.read(&mut header_size)?;
    let header_size = u64::from_be_bytes(header_size);
    let header: CompiledApplicationHeader = bincode::deserialize_from(reader.take(header_size))?;

    let component_buffers: Vec<_> = header
        .components
        .into_iter()
        .map(ComponentType::from)
        .collect();

    Ok(())
}

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only supported on 64 bit Operating Systems");
