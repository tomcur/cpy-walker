use num_bigint::BigInt;
use std::marker::PhantomData;

use crate::error::{Error, Result};
use crate::memory::Memory;

pub const PY_SIZE_T: usize = std::mem::size_of::<usize>();

pub enum Type {
    Type,
    Object,
    None,
    Class,
    Instance,
    Bytes,
    String,
    Unicode,
    Tuple,
    List,
    Dict,
    Bool,
    Int,
    Float,
}

/// Implementors of this trait collect together specific CPython object
/// implementations. This allows mixing and matching of implementations. Usually
/// this trait will be implemented by a marker type.
pub trait Interpreter: Copy + Clone + std::fmt::Debug {
    type TypedObject: TypedObject<Self>;
    type TypeObject: TypeObject<Self> + TryDeref;
    type Object: Object<Self> + TryDeref;
    type VarObject: VarObject<Self> + TryDeref;
    type ClassObject: ClassObject<Self> + TryDeref;
    type InstanceObject: InstanceObject<Self> + TryDeref;
    type NoneObject: NoneObject<Self> + TryDeref;
    type BytesObject: BytesObject<Self>;
    type StringObject: StringObject<Self> + TryDeref;
    type UnicodeObject: UnicodeObject<Self> + TryDeref;
    type TupleObject: TupleObject<Self> + TryDeref;
    type ListObject: ListObject<Self> + TryDeref;
    type DictEntry: DictEntry<Self>;
    type DictObject: DictObject<Self> + TryDeref;
    type BoolObject: BoolObject<Self> + TryDeref;
    type IntObject: IntObject<Self> + TryDeref;
    type FloatObject: FloatObject<Self> + TryDeref;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pointer {
    address: usize,
}

impl std::ops::Add<usize> for Pointer {
    type Output = Self;

    fn add(mut self, other: usize) -> Self {
        self.address += other;
        self
    }
}

impl std::ops::Add<isize> for Pointer {
    type Output = Self;

    fn add(mut self, other: isize) -> Self {
        self.address = (self.address as isize + other) as usize;
        self
    }
}

impl Pointer {
    pub const SIZE: usize = PY_SIZE_T;

    pub fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn get_usize(&self, mem: &impl Memory) -> Result<usize> {
        mem.get_usize(self.address)
    }

    pub fn deref_c_str(&self, mem: &impl Memory, max_length: Option<usize>) -> Result<String> {
        mem.get_c_str(self.address_checked()?, max_length)
    }

    pub fn address(&self) -> usize {
        self.address
    }

    pub fn null(&self) -> bool {
        self.address == 0
    }

    pub fn try_deref_me<O: TryDeref>(&self, mem: &impl Memory) -> Result<O> {
        if self.null() {
            return Err(Error::NullPointer);
        }
        O::try_deref(mem, *self)
    }

    pub fn address_checked(&self) -> Result<usize> {
        if self.null() {
            Err(Error::NullPointer)
        } else {
            Ok(self.address())
        }
    }
}

impl TryDeref for Pointer {
    fn try_deref(mem: &impl Memory, pointer: Self) -> Result<Self> {
        Ok(Self {
            address: pointer.get_usize(mem)?,
        })
    }
}

pub trait TypedObject<I: Interpreter> {
    fn object_type(&self) -> Type;
    fn as_type(self) -> Option<I::TypeObject>;
    fn as_object(self) -> Option<(I::TypeObject, I::Object)>;
    fn as_none(self) -> Option<I::NoneObject>;
    fn as_class(self) -> Option<I::ClassObject>;
    fn as_instance(self) -> Option<I::InstanceObject>;
    fn as_bytes(self) -> Option<I::BytesObject>;
    fn as_string(self) -> Option<I::StringObject>;
    fn as_unicode(self) -> Option<I::UnicodeObject>;
    fn as_tuple(self) -> Option<I::TupleObject>;
    fn as_list(self) -> Option<I::ListObject>;
    fn as_dict(self) -> Option<I::DictObject>;
    fn as_bool(self) -> Option<I::BoolObject>;
    fn as_int(self) -> Option<I::IntObject>;
    fn as_float(self) -> Option<I::FloatObject>;
}

pub trait TryDeref: Sized {
    fn try_deref(mem: &impl Memory, pointer: Pointer) -> Result<Self>;
}

pub trait TypeObject<I: Interpreter> {
    fn to_var_object(&self) -> I::VarObject;
    fn name(&self) -> &str;
    fn tp_basicsize(&self) -> isize;
    fn tp_itemsize(&self) -> isize;
    fn tp_dictoffset(&self) -> isize;
    fn downcast(&self, mem: &impl Memory, object: I::Object) -> Result<I::TypedObject>;
}

pub trait Object<I: Interpreter<Object = Self>> {
    fn me(&self) -> Pointer;
    fn ob_type(&self, mem: &impl Memory) -> Result<I::TypeObject>;
    fn ob_type_pointer(&self) -> Pointer;
    fn attributes(&self, mem: &impl Memory) -> Result<Option<I::DictObject>>;

    fn downcast(self, mem: &impl Memory) -> Result<I::TypedObject>
    where
        Self: Sized,
    {
        self.ob_type(mem)?.downcast(mem, self as I::Object)
    }
}

pub trait VarObject<I: Interpreter<VarObject = Self>> {
    fn to_object(&self) -> I::Object;
    fn ob_size(&self) -> isize;
    fn attributes(&self, mem: &impl Memory) -> Result<Option<I::DictObject>>;
}

pub trait ClassObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn name(&self) -> &str;
    fn bases(&self, mem: &impl Memory) -> Result<Option<I::ClassObject>>;
}

pub trait InstanceObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn class(&self, mem: &impl Memory) -> Result<I::ClassObject>;
    fn attributes(&self, mem: &impl Memory) -> Result<I::DictObject>;
}

pub trait NoneObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
}

pub trait BytesObject<I: Interpreter> {
    fn to_var_object(&self) -> I::VarObject;
    fn read(&self, mem: &impl Memory) -> Result<Vec<u8>>;
}

pub trait StringObject<I: Interpreter> {
    fn to_var_object(&self) -> I::VarObject;
    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>>;

    fn read(&self, mem: &impl Memory) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.read_bytes(mem)?).to_string())
    }
}

pub trait UnicodeObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>>;
    fn read(&self, mem: &impl Memory) -> Result<String>;
}

pub struct TupleItems<'a, I, M> {
    mem: &'a M,
    offset: Pointer,
    end_pointer: Pointer,
    _interp: PhantomData<I>,
}

impl<'a, I, M> TupleItems<'a, I, M> {
    pub fn new(mem: &'a M, offset: Pointer, length: usize) -> Self {
        Self {
            mem,
            offset,
            end_pointer: offset + length * std::mem::size_of::<usize>(),
            _interp: PhantomData,
        }
    }
}

impl<'a, I: Interpreter, M: Memory> Iterator for TupleItems<'a, I, M> {
    type Item = Result<I::Object>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset.address < self.end_pointer.address {
            let object = self
                .offset
                .try_deref_me(self.mem)
                .and_then(|pointer: Pointer| pointer.try_deref_me(self.mem));
            self.offset = self.offset + std::mem::size_of::<usize>();
            Some(object)
        } else {
            None
        }
    }
}

pub trait TupleObject<I: Interpreter> {
    fn to_var_object(&self) -> I::VarObject;
    fn items<'a, M: Memory>(&self, mem: &'a M) -> TupleItems<'a, I, M>;
}

pub struct ListItems<'a, I, M> {
    mem: &'a M,
    offset: Pointer,
    end_pointer: Pointer,
    _interp: PhantomData<I>,
}

impl<'a, I, M> ListItems<'a, I, M> {
    pub fn new(mem: &'a M, offset: Pointer, length: usize) -> Self {
        Self {
            mem,
            offset,
            end_pointer: offset + length * std::mem::size_of::<usize>(),
            _interp: PhantomData,
        }
    }
}

impl<'a, I: Interpreter, M: Memory> Iterator for ListItems<'a, I, M> {
    type Item = Result<I::Object>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset.address < self.end_pointer.address {
            let object = self
                .offset
                .try_deref_me(self.mem)
                .and_then(|pointer: Pointer| pointer.try_deref_me(self.mem));
            self.offset = self.offset + std::mem::size_of::<usize>();
            Some(object)
        } else {
            None
        }
    }
}

pub trait ListObject<I: Interpreter> {
    fn to_var_object(&self) -> I::VarObject;
    fn items<'a, M: Memory>(&self, mem: &'a M) -> ListItems<'a, I, M>;
}

pub trait DictEntry<I: Interpreter> {
    fn hash(&self) -> usize;
    fn key(&self) -> &I::Object;
    fn value(&self) -> &I::Object;
    fn take(self) -> (usize, I::Object, I::Object);
}

pub trait DictObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn entries(&self, mem: &impl Memory) -> Result<Vec<I::DictEntry>>;
}

pub trait BoolObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn value(&self) -> bool;
}

pub trait IntObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn read(&self, mem: &impl Memory) -> Result<BigInt>;
}

pub trait FloatObject<I: Interpreter> {
    fn to_object(&self) -> I::Object;
    fn value(&self) -> f64;
}
