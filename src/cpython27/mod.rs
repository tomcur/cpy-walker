use memoffset::offset_of;
use num_bigint::BigInt;
use std::convert::TryInto;
use std::marker::PhantomData;

use crate::error::Result;
use crate::interpreter::{
    BoolObject, BytesObject, ClassObject, DictEntry, DictObject, FloatObject, InstanceObject,
    IntObject, Interpreter, ListObject, NoneObject, Object, Pointer, StringObject, TryDeref,
    TupleObject, Type, TypeObject, TypedObject, UnicodeObject, VarObject, PY_SIZE_T,
};
use crate::memory::Memory;

mod bindings;

/// An interpreter marker type for decoding of CPython 2.7 memory.
#[derive(Debug, Copy, Clone)]
pub struct Cpython2_7;

impl Interpreter for Cpython2_7 {
    type TypedObject = PyTypedObject<Self>;
    type TypeObject = PyTypeObject<Self>;
    type Object = PyObject<Self>;
    type VarObject = PyVarObject<Self>;
    type NoneObject = PyNoneObject<Self>;
    type ClassObject = PyClassObject<Self>;
    type InstanceObject = PyInstanceObject<Self>;
    type BytesObject = PyBytesObject<Self>;
    type StringObject = PyStringObject<Self>;
    type UnicodeObject = PyUnicodeObject<Self>;
    type TupleObject = PyTupleObject<Self>;
    type ListObject = PyListObject<Self>;
    type DictEntry = PyDictEntry<Self>;
    type DictObject = PyDictObject<Self>;
    type BoolObject = PyBoolObject<Self>;
    type IntObject = PyIntObject<Self>;
    type FloatObject = PyFloatObject<Self>;
}

/// An interpreter marker type for decoding of CPython 2.7 memory with small
/// string objects. It is not clear to me when this is the case, but it seems
/// some CPython 2.7-compatible targets have strings that are 4 bytes smaller.
#[derive(Debug, Copy, Clone)]
pub struct Cpython2_7SmallString;

impl Interpreter for Cpython2_7SmallString {
    type TypedObject = PyTypedObject<Self>;
    type TypeObject = PyTypeObject<Self>;
    type Object = PyObject<Self>;
    type VarObject = PyVarObject<Self>;
    type NoneObject = PyNoneObject<Self>;
    type ClassObject = PyClassObject<Self>;
    type InstanceObject = PyInstanceObject<Self>;
    type BytesObject = PyBytesObject<Self>;
    type StringObject = PySmallStringObject<Self>;
    type UnicodeObject = PyUnicodeObject<Self>;
    type TupleObject = PyTupleObject<Self>;
    type ListObject = PyListObject<Self>;
    type DictEntry = PyDictEntry<Self>;
    type DictObject = PyDictObject<Self>;
    type BoolObject = PyBoolObject<Self>;
    type IntObject = PyIntObject<Self>;
    type FloatObject = PyFloatObject<Self>;
}

#[derive(Clone, Debug)]
pub enum PyTypedObject<I: Interpreter> {
    Type(I::TypeObject),
    Object(I::TypeObject, I::Object),
    None(I::NoneObject),
    Class(I::ClassObject),
    Instance(I::InstanceObject),
    Str(I::StringObject),
    Unicode(I::UnicodeObject),
    Tuple(I::TupleObject),
    List(I::ListObject),
    Dict(I::DictObject),
    Bool(I::BoolObject),
    Int(I::IntObject),
    Float(I::FloatObject),
}

// Hacky: this does not exist in Python 2.7.
pub struct PyBytesObject<I> {
    _interp: PhantomData<I>,
}
impl<I: Interpreter> BytesObject<I> for PyBytesObject<I> {
    fn to_var_object(&self) -> I::VarObject {
        unimplemented!("Bytes does not exist in Python 2.7")
    }
    fn read(&self, _mem: &impl Memory) -> Result<Vec<u8>> {
        unimplemented!("Bytes does not exist in Python 2.7")
    }
}

impl<I: Interpreter> TypedObject<I> for PyTypedObject<I> {
    fn object_type(&self) -> Type {
        match self {
            PyTypedObject::Type(_) => Type::Type,
            PyTypedObject::Object(_, _) => Type::Object,
            PyTypedObject::None(_) => Type::None,
            PyTypedObject::Class(_) => Type::Class,
            PyTypedObject::Instance(_) => Type::Instance,
            PyTypedObject::Str(_) => Type::String,
            PyTypedObject::Unicode(_) => Type::Unicode,
            PyTypedObject::Tuple(_) => Type::Tuple,
            PyTypedObject::List(_) => Type::List,
            PyTypedObject::Dict(_) => Type::Dict,
            PyTypedObject::Bool(_) => Type::Bool,
            PyTypedObject::Int(_) => Type::Int,
            PyTypedObject::Float(_) => Type::Float,
        }
    }

    fn as_type(self) -> Option<I::TypeObject> {
        if let PyTypedObject::Type(object) = self {
            Some(object)
        } else {
            None
        }
    }

    fn as_object(self) -> Option<(I::TypeObject, I::Object)> {
        if let PyTypedObject::Object(object_type, object) = self {
            Some((object_type, object))
        } else {
            None
        }
    }
    fn as_none(self) -> Option<I::NoneObject> {
        if let PyTypedObject::None(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_class(self) -> Option<I::ClassObject> {
        if let PyTypedObject::Class(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_instance(self) -> Option<I::InstanceObject> {
        if let PyTypedObject::Instance(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_bytes(self) -> Option<I::BytesObject> {
        None
    }
    fn as_string(self) -> Option<I::StringObject> {
        if let PyTypedObject::Str(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_unicode(self) -> Option<I::UnicodeObject> {
        if let PyTypedObject::Unicode(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_tuple(self) -> Option<I::TupleObject> {
        if let PyTypedObject::Tuple(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_list(self) -> Option<I::ListObject> {
        if let PyTypedObject::List(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_dict(self) -> Option<I::DictObject> {
        if let PyTypedObject::Dict(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_bool(self) -> Option<I::BoolObject> {
        if let PyTypedObject::Bool(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_int(self) -> Option<I::IntObject> {
        if let PyTypedObject::Int(object) = self {
            Some(object)
        } else {
            None
        }
    }
    fn as_float(self) -> Option<I::FloatObject> {
        if let PyTypedObject::Float(object) = self {
            Some(object)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct PyTypeObject<I> {
    me: Pointer,
    object: bindings::PyTypeObject,
    name: String,
    _interp: PhantomData<I>,
}

pub const PY_TYPE_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyTypeObject>();

impl<I: Interpreter> TryDeref for PyTypeObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_TYPE_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_TYPE_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        let type_object = unsafe { std::mem::transmute(b) };

        Ok(Self {
            me: pointer,
            object: type_object,
            name: Pointer::new(type_object.tp_name as usize).deref_c_str(mem, Some(1_000))?,
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter> TypeObject<I> for PyTypeObject<I>
where
    I: Interpreter<TypeObject = Self, TypedObject = PyTypedObject<I>, VarObject = PyVarObject<I>>,
{
    fn to_var_object(&self) -> I::VarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
            _interp: PhantomData,
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

    fn downcast(&self, mem: &impl Memory, object: I::Object) -> Result<I::TypedObject> {
        let typed = match self.name.as_str() {
            "type" => PyTypedObject::Type(object.me().try_deref_me(mem)?),
            "NoneType" => PyTypedObject::None(object.me().try_deref_me(mem)?),
            "classobj" => PyTypedObject::Class(object.me().try_deref_me(mem)?),
            "instance" => PyTypedObject::Instance(object.me().try_deref_me(mem)?),
            "str" => PyTypedObject::Str(object.me().try_deref_me(mem)?),
            "unicode" => PyTypedObject::Unicode(object.me().try_deref_me(mem)?),
            "tuple" => PyTypedObject::Tuple(object.me().try_deref_me(mem)?),
            "list" => PyTypedObject::List(object.me().try_deref_me(mem)?),
            "dict" => PyTypedObject::Dict(object.me().try_deref_me(mem)?),
            "bool" => PyTypedObject::Bool(object.me().try_deref_me(mem)?),
            "int" => PyTypedObject::Int(object.me().try_deref_me(mem)?),
            "float" => PyTypedObject::Float(object.me().try_deref_me(mem)?),
            _ => PyTypedObject::Object((*self).clone(), object),
        };

        Ok(typed)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyObject<I> {
    me: Pointer,
    object: bindings::PyObject,
    _interp: std::marker::PhantomData<I>,
}

const PY_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyObject>();

impl<I> TryDeref for PyObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: std::marker::PhantomData,
        })
    }
}

impl<I: Interpreter<Object = Self>> Object<I> for PyObject<I> {
    fn me(&self) -> Pointer {
        self.me
    }

    fn ob_type(&self, mem: &impl Memory) -> Result<I::TypeObject> {
        self.ob_type_pointer().try_deref_me(mem)
    }

    fn ob_type_pointer(&self) -> Pointer {
        Pointer::new(self.object.ob_type as usize)
    }

    fn attributes(&self, mem: &impl Memory) -> Result<Option<I::DictObject>> {
        let dictoffset = self.ob_type(mem)?.tp_dictoffset();

        if dictoffset == 0 {
            Ok(None)
        } else {
            let dict_ptr: Pointer = (self.me + dictoffset).try_deref_me(mem)?;
            Ok(Some(dict_ptr.try_deref_me(mem)?))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyVarObject<I> {
    me: Pointer,
    object: bindings::PyVarObject,
    _interp: std::marker::PhantomData<I>,
}

pub const PY_VAR_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyVarObject>();

impl<I> TryDeref for PyVarObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_VAR_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_VAR_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: std::marker::PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>, VarObject = Self>> VarObject<I> for PyVarObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn ob_size(&self) -> isize {
        self.object.ob_size
    }

    fn attributes(&self, mem: &impl Memory) -> Result<Option<I::DictObject>> {
        let tp: I::TypeObject = self.to_object().ob_type(mem)?;
        let dictoffset = tp.tp_dictoffset();

        if dictoffset == 0 {
            Ok(None)
        } else {
            let dict_ptr: Pointer = if dictoffset < 0 {
                (self.me + dictoffset).try_deref_me(mem)?
            } else {
                let offset = (tp.tp_basicsize()
                    + self.ob_size().abs() * tp.tp_itemsize()
                    + dictoffset) as usize;
                // Align to full word.
                let offset = (offset + PY_SIZE_T - 1) / PY_SIZE_T * PY_SIZE_T;
                (self.me + offset).try_deref_me(mem)?
            };
            Ok(Some(dict_ptr.try_deref_me(mem)?))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyNoneObject<I> {
    me: Pointer,
    object: bindings::PyObject,
    _interp: PhantomData<I>,
}

pub const PY_NONE_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyObject>();

impl<I> TryDeref for PyNoneObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_NONE_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_NONE_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> NoneObject<I> for PyNoneObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct PyClassObject<I> {
    me: Pointer,
    object: python27_sys::PyClassObject,
    name: String,
    _interp: PhantomData<I>,
}

impl<I> std::fmt::Debug for PyClassObject<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PyClassObject")
            .field("me", &self.me)
            .field("name", &self.name)
            .finish()
    }
}

pub const PY_CLASS_OBJECT_SIZE: usize = std::mem::size_of::<python27_sys::PyClassObject>();

impl<I: Interpreter> TryDeref for PyClassObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_CLASS_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_CLASS_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        let class_object: python27_sys::PyClassObject = unsafe { std::mem::transmute(b) };
        let class_name_string: I::StringObject =
            Pointer::new(class_object.cl_name as usize).try_deref_me(mem)?;

        Ok(Self {
            me: pointer,
            object: class_object,
            name: class_name_string.read(mem)?,
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> ClassObject<I> for PyClassObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type as *mut bindings::_typeobject,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn bases(&self, mem: &impl Memory) -> Result<Option<I::ClassObject>> {
        let class_ptr: Pointer = Pointer::new(self.object.cl_bases as usize);
        if class_ptr.null() {
            Ok(None)
        } else {
            Ok(Some(class_ptr.try_deref_me(mem)?))
        }
    }
}

#[derive(Copy, Clone)]
pub struct PyInstanceObject<I> {
    me: Pointer,
    object: python27_sys::PyInstanceObject,
    _interp: PhantomData<I>,
}

impl<I> std::fmt::Debug for PyInstanceObject<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PyInstanceObject")
            .field("me", &self.me)
            .finish()
    }
}

pub const PY_INSTANCE_OBJECT_SIZE: usize = std::mem::size_of::<python27_sys::PyInstanceObject>();

impl<I> TryDeref for PyInstanceObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_INSTANCE_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_INSTANCE_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> InstanceObject<I> for PyInstanceObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type as *mut bindings::_typeobject,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn class(&self, mem: &impl Memory) -> Result<I::ClassObject> {
        let class_ptr: Pointer = Pointer::new(self.object.in_class as usize);
        class_ptr.try_deref_me(mem)
    }

    fn attributes(&self, mem: &impl Memory) -> Result<I::DictObject> {
        let dict_ptr: Pointer = Pointer::new(self.object.in_dict as usize);
        dict_ptr.try_deref_me(mem)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyStringObject<I> {
    me: Pointer,
    object: bindings::PyStringObject,
    _interp: PhantomData<I>,
}

pub const PY_STRING_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyStringObject>();

impl<I> TryDeref for PyStringObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_STRING_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_STRING_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<VarObject = PyVarObject<I>>> StringObject<I> for PyStringObject<I> {
    fn to_var_object(&self) -> I::VarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>> {
        mem.get_vec(
            (self.me + offset_of!(bindings::PyStringObject, ob_sval)).address(),
            self.object.ob_size as usize,
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PySmallStringObject<I> {
    me: Pointer,
    object: bindings::PyStringObject,
    _interp: PhantomData<I>,
}

impl<I> TryDeref for PySmallStringObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_STRING_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_STRING_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<VarObject = PyVarObject<I>>> StringObject<I> for PySmallStringObject<I> {
    fn to_var_object(&self) -> I::VarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
            _interp: std::marker::PhantomData,
        }
    }

    // The - 4 seems wrong, but at least one of the Python 2.7 targets requires
    // this.
    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>> {
        mem.get_vec(
            (self.me + (offset_of!(bindings::PyStringObject, ob_sval) - 4)).address(),
            self.object.ob_size as usize,
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PyUnicodeObject<I> {
    me: Pointer,
    object: bindings::PyStringObject, // Hacky, we should get PyUnicodeObject in the bindings.
    _interp: PhantomData<I>,
}

pub const PY_UNICODE_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyStringObject>();

impl<I> PyUnicodeObject<I> {
    pub fn size(&self) -> isize {
        self.object.ob_size
    }
}

impl<I> TryDeref for PyUnicodeObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_UNICODE_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_UNICODE_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> UnicodeObject<I> for PyUnicodeObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
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

        Ok(String::from_utf16_lossy(&bytes))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyTupleObject<I> {
    me: Pointer,
    object: bindings::PyTupleObject,
    _interp: PhantomData<I>,
}

pub const PY_TUPLE_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyTupleObject>();

impl<I> TryDeref for PyTupleObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_TUPLE_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_TUPLE_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>, VarObject = PyVarObject<I>>> TupleObject<I>
    for PyTupleObject<I>
{
    fn to_var_object(&self) -> I::VarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn items(&self, mem: &impl Memory) -> Result<Vec<I::Object>> {
        let pointer =
            Pointer::new((&self.object.ob_item as *const *mut bindings::PyObject) as usize);

        let size = self.object.ob_size as usize;

        let mut items = Vec::with_capacity(size);
        for idx in 0..size {
            let object: I::Object = (pointer + idx * Pointer::SIZE).try_deref_me(mem)?;
            items.push(object)
        }

        Ok(items)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyListObject<I> {
    me: Pointer,
    object: bindings::PyListObject,
    _interp: PhantomData<I>,
}

pub const PY_LIST_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyListObject>();

impl<I> TryDeref for PyListObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_LIST_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_LIST_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>, VarObject = PyVarObject<I>>> ListObject<I>
    for PyListObject<I>
{
    fn to_var_object(&self) -> I::VarObject {
        PyVarObject {
            me: self.me,
            object: bindings::PyVarObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
                ob_size: self.object.ob_size,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn items(&self, mem: &impl Memory) -> Result<Vec<I::Object>> {
        let list_pointer = Pointer::new(self.object.ob_item as usize);
        let size = self.object.ob_size as usize;

        let mut items = Vec::with_capacity(size);
        for idx in 0..size {
            let object_pointer: Pointer = (list_pointer + idx * Pointer::SIZE).try_deref_me(mem)?;
            let object = object_pointer.try_deref_me(mem)?;
            items.push(object)
        }

        Ok(items)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyDictEntry<I> {
    hash: usize,
    key: PyObject<I>,
    value: PyObject<I>,
}

impl<I: Interpreter<Object = PyObject<I>>> DictEntry<I> for PyDictEntry<I> {
    fn hash(&self) -> usize {
        self.hash
    }

    fn key(&self) -> &I::Object {
        &self.key
    }

    fn value(&self) -> &I::Object {
        &self.value
    }

    fn take(self) -> (usize, I::Object, I::Object) {
        (self.hash, self.key, self.value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyDictObject<I> {
    me: Pointer,
    object: bindings::PyDictObject,
    _interp: PhantomData<I>,
}

pub const PY_DICT_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyDictObject>();

impl<I> PyDictObject<I> {
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

impl<I> TryDeref for PyDictObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_DICT_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_DICT_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>, DictEntry = PyDictEntry<I>>> DictObject<I>
    for PyDictObject<I>
{
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: PhantomData,
        }
    }

    fn entries(&self, mem: &impl Memory) -> Result<Vec<I::DictEntry>> {
        const ENTRY_SIZE: usize = std::mem::size_of::<bindings::PyDictEntry>();

        let table_addr: Pointer = Pointer::new(self.object.ma_table as usize);

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

            let key_pointer = Pointer::new(entry.me_key as usize);
            let value_pointer = Pointer::new(entry.me_value as usize);

            if key_pointer.null() || value_pointer.null() {
                continue;
            }

            entries.push(PyDictEntry {
                hash: entry.me_hash as usize,
                key: key_pointer.try_deref_me(mem)?,
                value: value_pointer.try_deref_me(mem)?,
            });
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyBoolObject<I> {
    me: Pointer,
    object: bindings::PyIntObject,
    _interp: PhantomData<I>,
}

pub const PY_BOOL_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyIntObject>();

impl<I> TryDeref for PyBoolObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_BOOL_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_BOOL_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> BoolObject<I> for PyBoolObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn value(&self) -> bool {
        self.object.ob_ival != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyIntObject<I> {
    me: Pointer,
    object: bindings::PyIntObject,
    _interp: PhantomData<I>,
}

pub const PY_INT_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyIntObject>();

impl<I> TryDeref for PyIntObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_INT_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_INT_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> IntObject<I> for PyIntObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn read(&self, _mem: &impl Memory) -> Result<BigInt> {
        Ok(self.object.ob_ival.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PyFloatObject<I> {
    me: Pointer,
    object: bindings::PyFloatObject,
    _interp: PhantomData<I>,
}

pub const PY_FLOAT_OBJECT_SIZE: usize = std::mem::size_of::<bindings::PyFloatObject>();

impl<I> TryDeref for PyFloatObject<I> {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self> {
        let b: [u8; PY_FLOAT_OBJECT_SIZE] = mem
            .get_vec(pointer.address(), PY_FLOAT_OBJECT_SIZE)?
            .try_into()
            .expect("const size");

        Ok(Self {
            me: pointer,
            object: unsafe { std::mem::transmute(b) },
            _interp: PhantomData,
        })
    }
}

impl<I: Interpreter<Object = PyObject<I>>> FloatObject<I> for PyFloatObject<I> {
    fn to_object(&self) -> I::Object {
        PyObject {
            me: self.me,
            object: bindings::PyObject {
                ob_refcnt: self.object.ob_refcnt,
                ob_type: self.object.ob_type,
            },
            _interp: std::marker::PhantomData,
        }
    }

    fn value(&self) -> f64 {
        self.object.ob_fval
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    use super::*;
    use crate::walker::{walk, DataPointer, DecodedData};

    #[test]
    fn works() -> std::result::Result<(), anyhow::Error> {
        let child = Command::new(
            [env!("CARGO_MANIFEST_DIR"), "test-programs", "python27.py"]
                .iter()
                .collect::<PathBuf>(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

        let pid = child.id();
        let stdout = child.stdout.unwrap();

        let mut line = String::new();
        BufReader::new(stdout).read_line(&mut line)?;
        let pointer: usize = line.trim().parse().expect("memory address");

        let mem = crate::connect(pid as i32)?;
        let ptr = Pointer::new(pointer);

        let graph = walk::<Cpython2_7, _>(&mem, ptr);

        if let Some(DecodedData::List(list)) = graph.get(&DataPointer(pointer)) {
            assert_eq!(list.len(), 3);
            match graph.get(&list[0]) {
                Some(&DecodedData::String(ref str)) => assert_eq!(str, "hello world"),
                _ => bail!("Expected a string"),
            }
            match graph.get(&list[1]) {
                Some(&DecodedData::Int(ref int)) => assert_eq!(int, &num_bigint::BigInt::from(42)),
                _ => bail!("Expected an int"),
            }
            match graph.get(&list[2]) {
                Some(&DecodedData::Instance {
                    ref instance_class_name,
                    ref attributes,
                    ..
                }) => {
                    assert_eq!(instance_class_name, "Something");
                    match attributes
                        .get("anything")
                        .and_then(|pointer| graph.get(pointer))
                    {
                        Some(DecodedData::String(str)) => assert_eq!(str, "I'm here!"),
                        _ => bail!("Expected an attribute"),
                    }
                }
                _ => bail!("Expected an instance"),
            }
        } else {
            bail!("Expected a list")
        }

        Ok(())
    }
}
