use crate::error::{Error, Result};
use crate::memory::Memory;

pub enum Type {
    Type,
    Object,
    Bytes,
    String,
    Unicode,
    Tuple,
    List,
    Dict,
}

pub trait TypedObject {
    type TypeObject: TypeObject<Object = Self::Object>;
    type Object: Object<TypeObject = Self::TypeObject>;
    type BytesObject: BytesObject;
    type StringObject: StringObject;
    type UnicodeObject: UnicodeObject<Object = Self::Object>;
    type TupleObject: TupleObject<Object = Self::Object>;
    type ListObject: ListObject<Object = Self::Object>;
    type DictObject: DictObject<Object = Self::Object>;

    fn object_type(&self) -> Type;
    fn as_type(self) -> Option<Self::TypeObject>;
    fn as_object(self) -> Option<(Self::TypeObject, Self::Object)>;
    fn as_bytes(self) -> Option<Self::BytesObject>;
    fn as_string(self) -> Option<Self::StringObject>;
    fn as_unicode(self) -> Option<Self::UnicodeObject>;
    fn as_tuple(self) -> Option<Self::TupleObject>;
    fn as_list(self) -> Option<Self::ListObject>;
    fn as_dict(self) -> Option<Self::DictObject>;
}

pub trait TryDeref: Sized {
    type Pointer: Pointer;
    fn try_deref(mem: &impl Memory, pointer: Self::Pointer) -> Result<Self>;
}

pub trait Pointer {
    fn address(&self) -> usize;
    fn null(&self) -> bool;
    fn try_deref<O: TryDeref<Pointer = Self>>(&self, mem: &impl Memory) -> Result<O>;
    fn address_checked(&self) -> Result<usize> {
        if self.null() {
            Err(Error::NullPointer)
        } else {
            Ok(self.address())
        }
    }
}

pub trait TypeObject {
    type Object: Object<TypeObject = Self>;
    type VarObject: VarObject<Object = Self::Object>;
    type TypedObject: TypedObject<TypeObject = Self, Object = Self::Object>;

    fn to_var_object(&self) -> Self::VarObject;
    fn name(&self) -> &str;
    fn tp_basicsize(&self) -> isize;
    fn tp_itemsize(&self) -> isize;
    fn tp_dictoffset(&self) -> isize;
    fn downcast(&self, mem: &impl Memory, object: Self::Object) -> Result<Self::TypedObject>;
}

pub trait Object {
    type Pointer: Pointer;
    type TypeObject: TypeObject<Object = Self>;
    type DictObject: DictObject<Object = Self>;
    fn me(&self) -> Self::Pointer;
    fn ob_type(&self, mem: &impl Memory) -> Result<Self::TypeObject>;
    fn ob_type_pointer(&self) -> Self::Pointer;
    fn attributes(&self, mem: &impl Memory) -> Result<Option<Self::DictObject>>;

    fn downcast(
        self,
        mem: &impl Memory,
    ) -> Result<<<Self as Object>::TypeObject as TypeObject>::TypedObject>
    where
        Self: Sized,
    {
        self.ob_type(mem)?.downcast(mem, self)
    }
}

pub trait VarObject {
    type Object: Object;
    type DictObject: DictObject;
    fn to_object(&self) -> Self::Object;
    fn ob_size(&self) -> isize;
    fn attributes(&self, mem: &impl Memory) -> Result<Option<Self::DictObject>>;
}

pub trait BytesObject {
    type VarObject: VarObject;
    fn to_var_object(&self) -> Self::VarObject;
    fn read(&self, mem: &impl Memory) -> Result<Vec<u8>>;
}

pub trait StringObject {
    type VarObject: VarObject;
    fn to_var_object(&self) -> Self::VarObject;
    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>>;

    fn read(&self, mem: &impl Memory) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.read_bytes(mem)?).to_string())
    }
}

pub trait UnicodeObject {
    type Object: Object;
    fn to_object(&self) -> Self::Object;
    fn read_bytes(&self, mem: &impl Memory) -> Result<Vec<u8>>;
    fn read(&self, mem: &impl Memory) -> Result<String>;
}

pub trait TupleObject {
    type Object: Object;
    type VarObject: VarObject;
    fn to_var_object(&self) -> Self::VarObject;
    fn items(&self, mem: &impl Memory) -> Result<Vec<Self::Object>>;
}

pub trait ListObject {
    type Object: Object;
    type VarObject: VarObject;
    fn to_var_object(&self) -> Self::VarObject;
    fn items(&self, mem: &impl Memory) -> Result<Vec<Self::Object>>;
}

pub trait DictEntry {
    type Object: Object;
    fn hash(&self) -> usize;
    fn key(&self) -> &Self::Object;
    fn value(&self) -> &Self::Object;
    fn take(self) -> (usize, Self::Object, Self::Object);
}

pub trait DictObject {
    type Object: Object;
    type DictEntry: DictEntry<Object = Self::Object>;
    fn to_object(&self) -> Self::Object;
    fn entries(&self, mem: &impl Memory) -> Result<Vec<Self::DictEntry>>;
}
