use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AnyValue {
    Int(isize),
    Float(f32),
    String(String)
}