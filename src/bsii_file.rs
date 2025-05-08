use std::collections::HashMap;

pub struct BsiiFile<'a> {
    pub(crate) header: &'a [u8], // BSII,
    pub(crate) version: u32,
    pub(crate) prototypes: HashMap<u32, Prototype<'a>>,
    pub data_blocks: Vec<DataBlock<'a>>,
}

pub struct Prototype<'a> {
    // valid prototypes only
    pub(crate) id: u32,
    pub name: &'a str,
    pub value_prototypes: Vec<ValuePrototype<'a>>,
}

#[derive(Debug)]
pub struct ValuePrototype<'a> {
    pub(crate) type_id: u32,
    pub name: &'a str,
    // enum values are only used when type_id is 0x37
    pub enum_values: Option<HashMap<u32, &'a str>>,
}

pub struct DataBlock<'a> {
    pub type_id: u32,
    pub id: Id,
    pub data: Vec<DataValue<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum Id {
    Nameless(u64),
    Named(Vec<String>),
}

/// A "placement" is a tuple of 8 floats, according to
/// <https://modding.scssoft.com/wiki/Documentation/Engine/Units>
pub type Placement = (f32, f32, f32, f32, f32, f32, f32, f32);

// TODO: Refactor this code so that singletons and vectors of different types
// are not duplicated
#[derive(PartialEq, Debug)]
pub enum DataValue<'a> {
    String(&'a str),
    StringArray(Vec<&'a str>),
    EncodedString(String),
    EncodedStringArray(Vec<String>),
    Float(f32),
    FloatArray(Vec<f32>),
    FloatVec2((f32, f32)),
    FloatVec3((f32, f32, f32)),
    FloatVec3Array(Vec<(f32, f32, f32)>),
    Int32Vec3((i32, i32, i32)),
    Int32Vec3Array(Vec<(i32, i32, i32)>),
    FloatVec4((f32, f32, f32, f32)),
    FloatVec4Array(Vec<(f32, f32, f32, f32)>),
    // Float Vec 7 for version 1 not supported
    FloatVec8(Placement),
    FloatVec8Array(Vec<Placement>),
    Int32(i32),
    Int32Array(Vec<i32>),
    UInt32(u32),
    UInt32Array(Vec<u32>),
    UInt16(u16),
    UInt16Array(Vec<u16>),
    Int64(i64),
    Int64Array(Vec<i64>),
    UInt64(u64),
    UInt64Array(Vec<u64>),
    Bool(bool),
    BoolArray(Vec<bool>),
    Enum(u32),
    Id(Id),
    IdArray(Vec<Id>),
}
