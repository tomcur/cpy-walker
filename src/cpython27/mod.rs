use memoffset::offset_of;
use num_bigint::BigInt;
use std::convert::TryInto;

use crate::error::{Error, Result};
use crate::interpreter::*;
use crate::memory::Memory;

mod bindings;

const PY_SIZE_T: usize = std::mem::size_of::<usize>();

#[derive(Clone, Debug)]
pub enum PyTypedObject {
    Type(PyTypeObject),
    Object(PyTypeObject, PyObject),
    None(PyNoneObject),
    Str(PyStringObject),
    Unicode(PyUnicodeObject),
    Tuple(PyTupleObject),
    List(PyListObject),
    Dict(PyDictObject),
    Int(PyIntObject),
    Float(PyFloatObject),
}

// Hacky: this does not exist in Python 2.7.
pub struct PyBytesObject;
impl BytesObject for PyBytesObject {
    type VarObject = PyVarObject;
    fn to_var_object(&self) -> Self::VarObject {
        unimplemented!("Bytes does not exist in Python 2.7")
    }
    fn read(&self, _mem: &impl Memory) -> Result<Vec<u8>> {
        unimplemented!("Bytes does not exist in Python 2.7")
    }
}

impl TypedObject for PyTypedObject {
    type TypeObject = PyTypeObject;
    type Object = PyObject;
    type NoneObject = PyNoneObject;
    type BytesObject = PyBytesObject;
    type StringObject = PyStringObject;
    type UnicodeObject = PyUnicodeObject;
    type TupleObject = PyTupleObject;
    type ListObject = PyListObject;
    type DictObject = PyDictObject;
    type IntObject = PyIntObject;
    type FloatObject = PyFloatObject;

    fn object_type(&self) -> Type {
        match self {
            PyTypedObject::Type(_) => Type::Type,
            PyTypedObject::Object(_, _) => Type::Object,
            PyTypedObject::None(_) => Type::None,
            PyTypedObject::Str(_) => Type::String,
            PyTypedObject::Unicode(_) => Type::Unicode,
            PyTypedObject::Tuple(_) => Type::Tuple,
            PyTypedObject::List(_) => Type::List,
            PyTypedObject::Dict(_) => Type::Dict,
            PyTypedObject::Int(_) => Type::Int,
            PyTypedObject::Float(_) => Type::Float,
        }
    }

    fn as_type(self) -> Option<Self::TypeObject> {
        if let PyTypedObject::Type(object) = self {
            Some(object)
        } else {
            None
        }
    }

    fn as_object(self) -> Option<(PyTypeObject, PyObject)> {
        if let PyTypedObject::Object(object_type, object) = self {
            Some((object_type, object))
        } else {
            None
        }
    }
    fn as_none(self) -> Option<Self::NoneObject> {
        if let PyTypedObject::None(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_bytes(self) -> Option<Self::BytesObject> {
        None
    }
    fn as_string(self) -> Option<Self::StringObject> {
        if let PyTypedObject::Str(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_unicode(self) -> Option<Self::UnicodeObject> {
        if let PyTypedObject::Unicode(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_tuple(self) -> Option<Self::TupleObject> {
        if let PyTypedObject::Tuple(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_list(self) -> Option<Self::ListObject> {
        if let PyTypedObject::List(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_dict(self) -> Option<Self::DictObject> {
        if let PyTypedObject::Dict(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_int(self) -> Option<Self::IntObject> {
        if let PyTypedObject::Int(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_float(self) -> Option<Self::FloatObject> {
        if let PyTypedObject::Float(object) = self {
            Some(object)
        } else {
            None
        }
    }
}

// impl TryDecode for TypedObject {
//    fn try_decode(self, mem: &impl Memory) -> Result<DecodedData> {
//        use TypedObject::*;
//
//        let data = match self {
//            Type(object) => object.name(),
//            _ => unimplemented!("asd"),
//        };
//
//        Ok(data)
//    }
//}

#[derive(Copy, Clone, Debug)]
pub struct PyPointer {
    address: usize,
}

impl std::ops::Add<usize> for PyPointer {
    type Output = Self;

    fn add(mut self, other: usize) -> Self {
        self.address += other;
        self
    }
}

impl std::ops::Add<isize> for PyPointer {
    type Output = Self;

    fn add(mut self, other: isize) -> Self {
        self.address = (self.address as isize + other) as usize;
        self
    }
}

impl PyPointer {
    pub const SIZE: usize = PY_SIZE_T;

    pub fn new(address: usize) -> Self {
        Self { address }
    }

    fn get_usize(&self, mem: &impl Memory) -> Result<usize> {
        mem.get_usize(self.address)
    }

    fn deref_c_str(&self, mem: &impl Memory, max_length: Option<usize>) -> Result<String> {
        mem.get_c_str(self.address_checked()?, max_length)
    }
}

impl TryDeref for PyPointer {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        Ok(Self {
            address: pointer.get_usize(mem)?,
        })
    }
}

impl Pointer for PyPointer {
    fn address(&self) -> usize {
        self.address
    }

    fn null(&self) -> bool {
        self.address == 0
    }

    fn try_deref<O: TryDeref<Pointer = PyPointer>>(&self, mem: &impl Memory) -> Result<O> {
        if self.null() {
            return Err(Error::NullPointer);
        }
        O::try_deref(mem, *self)
    }
}

#[derive(Clone, Debug)]
pub struct PyTypeObject {
    me: PyPointer,
    object: bindings::PyTypeObject,
    name: String,
}

impl PyTypeObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyTypeObject>();
}

impl TryDeref for PyTypeObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        let type_object = unsafe { std::mem::transmute(b) };

        Ok(Self {
            me: pointer,
            object: type_object,
            name: PyPointer::new(type_object.tp_name as usize).deref_c_str(mem, Some(1_000))?,
        })
    }
}

impl TypeObject for PyTypeObject {
    type Object = PyObject;
    type VarObject = PyVarObject;
    type TypedObject = PyTypedObject;

    fn to_var_object(&self) -> PyVarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn tp_basicsize(&self) -> isize {
        self.object.tp_basicsize
    }

    fn tp_itemsize(&self) -> isize {
        self.object.tp_itemsize
    }

    fn tp_dictoffset(&self) -> isize {
        self.object.tp_dictoffset
    }

    fn downcast(&self, mem: &impl Memory, object: Self::Object) -> Result<Self::TypedObject> {
        let typed = match self.name.as_str() {
            "type" => PyTypedObject::Type(object.me.try_deref(mem)?),
            "str" => PyTypedObject::Str(object.me.try_deref(mem)?),
            "unicode" => PyTypedObject::Unicode(object.me.try_deref(mem)?),
            "tuple" => PyTypedObject::Tuple(object.me.try_deref(mem)?),
            "list" => PyTypedObject::List(object.me.try_deref(mem)?),
            "dict" => PyTypedObject::Dict(object.me.try_deref(mem)?),
            "int" => PyTypedObject::Int(object.me.try_deref(mem)?),
            "float" => PyTypedObject::Float(object.me.try_deref(mem)?),
            _ => PyTypedObject::Object(self.clone(), object),
        };

        Ok(typed)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyObject {
    me: PyPointer,
    object: bindings::PyObject,
}

impl PyObject {
    const SIZE: usize = std::mem::size_of::<bindings::PyObject>();
}

impl TryDeref for PyObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl Object for PyObject {
    type Pointer = PyPointer;
    type TypeObject = PyTypeObject;
    type DictObject = PyDictObject;

    fn me(&self) -> Self::Pointer {
        self.me
    }

    fn ob_type(&self, mem: &impl Memory) -> Result<Self::TypeObject> {
        self.ob_type_pointer().try_deref(mem)
    }

    fn ob_type_pointer(&self) -> Self::Pointer {
        PyPointer::new(self.object.ob_type as usize)
    }

    fn attributes(&self, mem: &impl Memory) -> Result<Option<Self::DictObject>> {
        let dictoffset = self.ob_type(mem)?.object.tp_dictoffset;

        if dictoffset == 0 {
            Ok(None)
        } else {
            let dict_ptr: PyPointer = (self.me + dictoffset).try_deref(mem)?;
            Ok(Some(dict_ptr.try_deref(mem)?))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyVarObject {
    me: PyPointer,
    object: bindings::PyVarObject,
}

impl PyVarObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyVarObject>();
}

impl TryDeref for PyVarObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl VarObject for PyVarObject {
    type Object = PyObject;
    type DictObject = PyDictObject;

    fn to_object(&self) -> Self::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }

    fn ob_size(&self) -> isize {
        self.object.ob_size
    }

    fn attributes(&self, mem: &impl Memory) -> Result<Option<Self::DictObject>> {
        let tp = self.to_object().ob_type(mem)?;
        let dictoffset = tp.tp_dictoffset();

        if dictoffset == 0 {
            Ok(None)
        } else {
            let dict_ptr: PyPointer = if dictoffset < 0 {
                (self.me + dictoffset).try_deref(mem)?
            } else {
                let offset = (tp.tp_basicsize()
                    + self.ob_size().abs() * tp.tp_itemsize()
                    + dictoffset) as usize;
                // Align to full word.
                let offset = (offset + PY_SIZE_T - 1) / PY_SIZE_T * PY_SIZE_T;
                (self.me + offset).try_deref(mem)?
            };
            Ok(Some(dict_ptr.try_deref(mem)?))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyNoneObject {
    me: PyPointer,
    object: bindings::PyObject,
}

impl PyNoneObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyObject>();
}

impl TryDeref for PyNoneObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl NoneObject for PyNoneObject {
    type Object = PyObject;

    fn to_object(&self) -> Self::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyStringObject {
    me: PyPointer,
    object: bindings::PyStringObject,
}

impl PyStringObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyStringObject>();
}

impl TryDeref for PyStringObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl StringObject for PyStringObject {
    type VarObject = PyVarObject;

    fn to_var_object(&self) -> PyVarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
        }
    }

    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>> {
        mem.get_vec(
            // TODO: The - 4 seems wrong.
            (self.me + (offset_of!(bindings::PyStringObject, ob_sval) - 4)).address(),
            self.object.ob_size as usize,
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyUnicodeObject {
    me: PyPointer,
    object: bindings::PyStringObject, // Hacky, we should get PyUnicodeObject in the bindings.
}

impl PyUnicodeObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyStringObject>();

    pub fn size(&self) -> isize {
        self.object.ob_size
    }
}

impl TryDeref for PyUnicodeObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl UnicodeObject for PyUnicodeObject {
    type Object = PyObject;

    fn to_object(&self) -> PyObject {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }

    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>> {
        mem.get_vec(
            (&self.object.ob_sval as *const [i8; 1]) as usize,
            self.object.ob_size as usize,
        )
    }

    fn read(&self, mem: &impl Memory) -> Result<String> {
        let bytes = mem.get_u16_vec(
            (&self.object.ob_sval as *const [i8; 1]) as usize,
            self.object.ob_size as usize,
        )?;

        Ok(String::from_utf16_lossy(&bytes).to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyTupleObject {
    me: PyPointer,
    object: bindings::PyTupleObject,
}

impl PyTupleObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyTupleObject>();
}

impl TryDeref for PyTupleObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl TupleObject for PyTupleObject {
    type Object = PyObject;
    type VarObject = PyVarObject;

    fn to_var_object(&self) -> PyVarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
        }
    }

    fn items(&self, mem: &impl Memory) -> Result<Vec<Self::Object>> {
        let pointer =
            PyPointer::new((&self.object.ob_item as *const *mut bindings::PyObject) as usize);

        let size = self.object.ob_size as usize;

        let mut items = Vec::with_capacity(size);
        for idx in 0..size {
            let object = (pointer + idx * PyPointer::SIZE).try_deref(mem)?;
            items.push(object)
        }

        Ok(items)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyListObject {
    me: PyPointer,
    object: bindings::PyListObject,
}

impl PyListObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyListObject>();
}

impl TryDeref for PyListObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl ListObject for PyListObject {
    type Object = PyObject;
    type VarObject = PyVarObject;

    fn to_var_object(&self) -> PyVarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
        }
    }

    fn items(&self, mem: &impl Memory) -> Result<Vec<Self::Object>> {
        let list_pointer = PyPointer::new(self.object.ob_item as usize);
        let size = self.object.ob_size as usize;

        let mut items = Vec::with_capacity(size);
        for idx in 0..size {
            let object_pointer: PyPointer =
                (list_pointer + idx * PyPointer::SIZE).try_deref(mem)?;
            let object = object_pointer.try_deref(mem)?;
            items.push(object)
        }

        Ok(items)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyDictEntry {
    hash: usize,
    key: PyObject,
    value: PyObject,
}

impl DictEntry for PyDictEntry {
    type Object = PyObject;

    fn hash(&self) -> usize {
        self.hash
    }

    fn key(&self) -> &PyObject {
        &self.key
    }

    fn value(&self) -> &PyObject {
        &self.value
    }

    fn take(self) -> (usize, Self::Object, Self::Object) {
        (self.hash, self.key, self.value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyDictObject {
    me: PyPointer,
    object: bindings::PyDictObject,
}

impl PyDictObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyDictObject>();

    pub fn fill(&self) -> isize {
        self.object.ma_fill
    }

    pub fn used(&self) -> isize {
        self.object.ma_used
    }

    pub fn mask(&self) -> isize {
        self.object.ma_mask
    }
}

impl TryDeref for PyDictObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl DictObject for PyDictObject {
    type Object = PyObject;
    type DictEntry = PyDictEntry;

    fn to_object(&self) -> PyObject {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }

    fn entries(&self, mem: &impl Memory) -> Result<Vec<Self::DictEntry>> {
        const ENTRY_SIZE: usize = std::mem::size_of::<bindings::PyDictEntry>();

        let table_addr: PyPointer = PyPointer::new(self.object.ma_table as usize);

        let mut slots = self.mask() as usize + 1;
        if slots >= 10_000 {
            tracing::warn!("dict too big");
            slots = 10_000;
        }

        let mut entries = Vec::new();
        for slot in 0..slots {
            let pointer = table_addr + slot * ENTRY_SIZE;

            let b: [u8; ENTRY_SIZE] = mem
                .get_vec(pointer.address(), ENTRY_SIZE)?
                .try_into()
                .expect("const size");

            let entry: bindings::PyDictEntry = unsafe { std::mem::transmute(b) };

            let key_pointer = PyPointer::new(entry.me_key as usize);
            let value_pointer = PyPointer::new(entry.me_value as usize);

            if key_pointer.null() || value_pointer.null() {
                continue;
            }

            entries.push(PyDictEntry {
                hash: entry.me_hash as usize,
                key: key_pointer.try_deref(mem)?,
                value: value_pointer.try_deref(mem)?,
            });
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyIntObject {
    me: PyPointer,
    object: bindings::PyIntObject,
}

impl PyIntObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyIntObject>();
}

impl TryDeref for PyIntObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl IntObject for PyIntObject {
    type Object = PyObject;

    fn to_object(&self) -> PyObject {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }

    fn read(&self, _mem: &impl Memory) -> Result<BigInt> {
        Ok(self.object.ob_ival.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyFloatObject {
    me: PyPointer,
    object: bindings::PyFloatObject,
}

impl PyFloatObject {
    pub const SIZE: usize = std::mem::size_of::<bindings::PyFloatObject>();
}

impl TryDeref for PyFloatObject {
    type Pointer = PyPointer;

    fn try_deref(mem: &impl Memory, pointer: PyPointer) -> Result<Self> {
        let b: [u8; Self::SIZE] = mem
            .get_vec(pointer.address(), Self::SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
        })
    }
}

impl FloatObject for PyFloatObject {
    type Object = PyObject;

    fn to_object(&self) -> PyObject {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
        }
    }

    fn value(&self) -> f64 {
        self.object.ob_fval
    }
}
