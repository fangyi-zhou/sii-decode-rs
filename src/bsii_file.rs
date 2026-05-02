//! Defines the BSII file format (binary SII file format)

use std::collections::HashMap;
use std::slice;

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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

/// A value paired with the prototype field that defines it.
pub struct DataField<'file, 'data> {
    pub prototype: &'data ValuePrototype<'file>,
    pub value: &'data DataValue<'file>,
}

/// Iterator over the fields in a data block.
pub struct DataFields<'file, 'data> {
    prototypes: slice::Iter<'data, ValuePrototype<'file>>,
    values: slice::Iter<'data, DataValue<'file>>,
}

impl<'file, 'data> Iterator for DataFields<'file, 'data> {
    type Item = DataField<'file, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        let prototype = self.prototypes.next()?;
        let value = self.values.next()?;
        Some(DataField { prototype, value })
    }
}

impl<'a> BsiiFile<'a> {
    /// Return the BSII file header bytes.
    pub fn header(&self) -> &'a [u8] {
        self.header
    }

    /// Return the BSII format version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Look up a prototype by numeric prototype ID.
    pub fn get_prototype(&self, id: u32) -> Option<&Prototype<'a>> {
        self.prototypes.get(&id)
    }

    /// Iterate over all prototypes.
    pub fn prototypes(&self) -> impl Iterator<Item = &Prototype<'a>> {
        self.prototypes.values()
    }

    /// Iterate over all data blocks.
    pub fn data_blocks(&self) -> impl Iterator<Item = &DataBlock<'a>> {
        self.data_blocks.iter()
    }

    /// Iterate over blocks whose prototype has the given name.
    pub fn blocks_by_prototype_name<'data>(
        &'data self,
        name: &'data str,
    ) -> impl Iterator<Item = &'data DataBlock<'a>> + 'data {
        self.data_blocks.iter().filter(move |block| {
            self.get_prototype(block.prototype_id)
                .is_some_and(|prototype| prototype.name == name)
        })
    }
}

impl<'a> Prototype<'a> {
    /// Return the numeric prototype ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Iterate over the prototype's field definitions.
    pub fn fields(&self) -> impl Iterator<Item = &ValuePrototype<'a>> {
        self.value_prototypes.iter()
    }

    /// Return the index of a field by name.
    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.value_prototypes
            .iter()
            .position(|field| field.name == name)
    }

    /// Look up a field definition by name.
    pub fn field(&self, name: &str) -> Option<&ValuePrototype<'a>> {
        self.field_index(name)
            .and_then(|index| self.value_prototypes.get(index))
    }
}

impl ValuePrototype<'_> {
    /// Return the BSII type ID for this field.
    pub fn type_id(&self) -> u32 {
        self.type_id
    }
}

impl<'a> DataBlock<'a> {
    /// Look up this block's prototype in the containing file.
    pub fn prototype<'data>(&self, file: &'data BsiiFile<'a>) -> Option<&'data Prototype<'a>> {
        file.get_prototype(self.prototype_id)
    }

    /// Iterate over this block's values paired with their field definitions.
    ///
    /// Returns `None` if the block references a missing prototype or if the
    /// number of values does not match the prototype definition.
    pub fn fields<'data>(&'data self, file: &'data BsiiFile<'a>) -> Option<DataFields<'a, 'data>> {
        let prototype = self.prototype(file)?;
        if prototype.value_prototypes.len() != self.data.len() {
            return None;
        }
        Some(DataFields {
            prototypes: prototype.value_prototypes.iter(),
            values: self.data.iter(),
        })
    }

    /// Look up a value by field name.
    pub fn field<'data>(
        &'data self,
        file: &'data BsiiFile<'a>,
        name: &str,
    ) -> Option<&'data DataValue<'a>> {
        let prototype = self.prototype(file)?;
        let index = prototype.field_index(name)?;
        self.data.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_inspect_prototypes_blocks_and_fields_safely() {
        let prototype = Prototype {
            id: 7,
            name: "delivery_log_entry",
            value_prototypes: vec![
                ValuePrototype {
                    type_id: 0x03,
                    name: "cargo",
                    enum_values: None,
                },
                ValuePrototype {
                    type_id: 0x25,
                    name: "revenue",
                    enum_values: None,
                },
            ],
        };
        let file = BsiiFile {
            header: b"BSII",
            version: 2,
            prototypes: HashMap::from([(prototype.id, prototype)]),
            data_blocks: vec![DataBlock {
                prototype_id: 7,
                id: Id::Nameless(1),
                data: vec![
                    DataValue::EncodedString("gravel".to_string()),
                    DataValue::Int32(12500),
                ],
            }],
        };

        assert_eq!(file.header(), b"BSII");
        assert_eq!(file.version(), 2);

        let prototype = file.get_prototype(7).unwrap();
        assert_eq!(prototype.id(), 7);
        assert_eq!(prototype.field("cargo").unwrap().type_id(), 0x03);
        assert_eq!(prototype.fields().count(), 2);

        let block = file
            .blocks_by_prototype_name("delivery_log_entry")
            .next()
            .unwrap();
        assert_eq!(
            block.field(&file, "cargo"),
            Some(&DataValue::EncodedString("gravel".to_string()))
        );
        assert_eq!(block.field(&file, "missing"), None);

        let fields = block.fields(&file).unwrap().collect::<Vec<_>>();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].prototype.name, "cargo");
        assert_eq!(
            fields[0].value,
            &DataValue::EncodedString("gravel".to_string())
        );
    }

    #[test]
    fn block_fields_rejects_mismatched_prototype_lengths() {
        let prototype = Prototype {
            id: 1,
            name: "short",
            value_prototypes: vec![],
        };
        let file = BsiiFile {
            header: b"BSII",
            version: 2,
            prototypes: HashMap::from([(prototype.id, prototype)]),
            data_blocks: vec![DataBlock {
                prototype_id: 1,
                id: Id::Nameless(1),
                data: vec![DataValue::Int32(1)],
            }],
        };

        assert!(file.data_blocks[0].fields(&file).is_none());
    }
}
