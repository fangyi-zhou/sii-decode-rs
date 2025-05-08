//! Defines the BSII file format (binary SII file format)

use std::collections::HashMap;

/// BSII file
///
/// The BSII file format is a binary format. The file begins with a 4 byte
/// "BSII" header, followed by a version number.
/// Then comes a list of prototypes for datablocks, which can be considered as definitions of data classes.
/// After that, there are data blocks, which are instances of the prototypes.
///
/// More details can be found in <https://github.com/TheLazyTomcat/SII_Decrypt/blob/master/Documents/Binary%20SII%20-%20Format.txt>
pub struct BsiiFile<'a> {
    pub(crate) header: &'a [u8], // BSII,
    pub(crate) version: u32,
    pub prototypes: HashMap<u32, Prototype<'a>>,
    pub data_blocks: Vec<DataBlock<'a>>,
}

/// A prototype contains the definition of a data block, with an ID, a name, and a list of definition of fields.
pub struct Prototype<'a> {
    // valid prototypes only
    pub(crate) id: u32,
    pub name: &'a str,
    pub value_prototypes: Vec<ValuePrototype<'a>>,
}

/// A value prototype is a definition of a field in a data block.
/// Each value has an type ID, a name.
/// If the type ID is 0x37, it means that the value is an enum, and a list of
/// enum values are additionally provided.
#[derive(Debug)]
pub struct ValuePrototype<'a> {
    pub(crate) type_id: u32,
    pub name: &'a str,
    // enum values are only used when type_id is 0x37
    pub enum_values: Option<HashMap<u32, &'a str>>,
}

/// A data block is an instance of a prototype.
/// It contains a prototype ID (corresponding to a prototype defined earlier),
/// an ID (to identify this data block), and a list of values (corresponding to
/// fields defined in the prototype).
pub struct DataBlock<'a> {
    pub prototype_id: u32,
    pub id: Id,
    pub data: Vec<DataValue<'a>>,
}

/// An ID to identify a data block.
/// It can be either a nameless ID (a 64 bit integer) or a named ID (consisting
/// of multiple parts).
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
/// A data value is a value of a field in a data block.
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
