mod value_type;
mod value_type_info;

pub use self::value_type::ValueType;
pub use self::value_type_info::ValueTypeInfo;
use crate::any::Any;
use crate::bytes::Bytes;
use crate::future::Future;
use crate::shared::Shared;
use std::any;
use std::rc::Rc;

use crate::hash::Hash;
use crate::vm::VmError;

/// The type of an object.
pub type Object<T> = crate::collections::HashMap<String, T>;

/// A helper type to deserialize arrays with different interior types.
///
/// This implements [FromValue], allowing it to be used as a return value from
/// a virtual machine.
///
/// [FromValue]: crate::FromValue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecTuple<I>(pub I);

/// A tuple with a well-defined type.
#[derive(Debug)]
pub struct TypedTuple {
    /// The type hash of the tuple.
    pub ty: Hash,
    /// Content of the tuple.
    pub tuple: Box<[Value]>,
}

/// An object with a well-defined type.
#[derive(Debug)]
pub struct TypedObject {
    /// The type hash of the object.
    pub ty: Hash,
    /// Content of the object.
    pub object: Object<Value>,
}

/// An entry on the stack.
#[derive(Debug, Clone)]
pub enum Value {
    /// The unit value.
    Unit,
    /// A boolean.
    Bool(bool),
    /// A single byte.
    Byte(u8),
    /// A character.
    Char(char),
    /// A number.
    Integer(i64),
    /// A float.
    Float(f64),
    /// A type.
    Type(Hash),
    /// A static string.
    /// The index is the index into the static string slot for the current unit.
    StaticString(Rc<str>),
    /// A UTF-8 string.
    String(Shared<String>),
    /// A byte string.
    Bytes(Shared<Bytes>),
    /// A vector containing any values.
    Vec(Shared<Vec<Value>>),
    /// A tuple.
    Tuple(Shared<Box<[Value]>>),
    /// An object.
    Object(Shared<Object<Value>>),
    /// A stored future.
    Future(Shared<Future>),
    /// An empty value indicating nothing.
    Option(Shared<Option<Value>>),
    /// A stored result in a slot.
    Result(Shared<Result<Value, Value>>),
    /// A tuple with a well-defined type.
    TypedTuple(Shared<TypedTuple>),
    /// An object with a well-defined type.
    TypedObject(Shared<TypedObject>),
    /// An external value.
    External(Shared<Any>),
}

impl Value {
    /// Cosntruct a value from a raw pointer.
    ///
    /// # Safety
    ///
    /// The returned value mustn't be used after it's been freed.
    pub unsafe fn from_ptr<T>(ptr: *const T) -> Self
    where
        T: any::Any,
    {
        Self::External(Shared::new(Any::from_ptr(ptr)))
    }

    /// Cosntruct a value from a raw mutable pointer.
    ///
    /// # Safety
    ///
    /// The returned value mustn't be used after it's been freed.
    pub unsafe fn from_mut_ptr<T>(ptr: *mut T) -> Self
    where
        T: any::Any,
    {
        Self::External(Shared::new(Any::from_mut_ptr(ptr)))
    }

    /// Try to coerce value reference into a result.
    #[inline]
    pub fn into_result(self) -> Result<Shared<Result<Value, Value>>, VmError> {
        match self {
            Self::Result(result) => Ok(result),
            actual => Err(VmError::ExpectedResult {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into an option.
    #[inline]
    pub fn into_option(self) -> Result<Shared<Option<Value>>, VmError> {
        match self {
            Self::Option(option) => Ok(option),
            actual => Err(VmError::ExpectedOption {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into a string.
    #[inline]
    pub fn into_string(self) -> Result<Shared<String>, VmError> {
        match self {
            Self::String(string) => Ok(string),
            actual => Err(VmError::ExpectedString {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into bytes.
    #[inline]
    pub fn into_bytes(self) -> Result<Shared<Bytes>, VmError> {
        match self {
            Self::Bytes(bytes) => Ok(bytes),
            actual => Err(VmError::ExpectedBytes {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into a vector.
    #[inline]
    pub fn into_vec(self) -> Result<Shared<Vec<Value>>, VmError> {
        match self {
            Self::Vec(vec) => Ok(vec),
            actual => Err(VmError::ExpectedVec {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into a tuple.
    #[inline]
    pub fn into_tuple(self) -> Result<Shared<Box<[Value]>>, VmError> {
        match self {
            Self::Tuple(tuple) => Ok(tuple),
            actual => Err(VmError::ExpectedTuple {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into an object.
    #[inline]
    pub fn into_object(self) -> Result<Shared<Object<Value>>, VmError> {
        match self {
            Self::Object(object) => Ok(object),
            actual => Err(VmError::ExpectedObject {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Try to coerce value reference into an external.
    #[inline]
    pub fn into_external(self) -> Result<Shared<Any>, VmError> {
        match self {
            Self::External(any) => Ok(any),
            actual => Err(VmError::ExpectedExternal {
                actual: actual.type_info()?,
            }),
        }
    }

    /// Get the type information for the current value.
    pub fn value_type(&self) -> Result<ValueType, VmError> {
        Ok(match self {
            Self::Unit => ValueType::Unit,
            Self::Bool(..) => ValueType::Bool,
            Self::Byte(..) => ValueType::Byte,
            Self::Char(..) => ValueType::Char,
            Self::Integer(..) => ValueType::Integer,
            Self::Float(..) => ValueType::Float,
            Self::StaticString(..) => ValueType::String,
            Self::String(..) => ValueType::String,
            Self::Bytes(..) => ValueType::Bytes,
            Self::Vec(..) => ValueType::Vec,
            Self::Tuple(..) => ValueType::Tuple,
            Self::Object(..) => ValueType::Object,
            Self::Type(..) => ValueType::Type,
            Self::Future(..) => ValueType::Future,
            Self::Result(..) => ValueType::Result,
            Self::Option(..) => ValueType::Option,
            Self::TypedTuple(typed_tuple) => {
                let ty = typed_tuple.get_ref()?.ty;
                ValueType::TypedTuple(ty)
            }
            Self::TypedObject(typed_object) => {
                let ty = typed_object.get_ref()?.ty;
                ValueType::TypedObject(ty)
            }
            Self::External(any) => ValueType::External(any.get_ref()?.type_id()),
        })
    }

    /// Get the type information for the current value.
    pub fn type_info(&self) -> Result<ValueTypeInfo, VmError> {
        Ok(match self {
            Self::Unit => ValueTypeInfo::Unit,
            Self::Bool(..) => ValueTypeInfo::Bool,
            Self::Byte(..) => ValueTypeInfo::Byte,
            Self::Char(..) => ValueTypeInfo::Char,
            Self::Integer(..) => ValueTypeInfo::Integer,
            Self::Float(..) => ValueTypeInfo::Float,
            Self::StaticString(..) => ValueTypeInfo::String,
            Self::String(..) => ValueTypeInfo::String,
            Self::Bytes(..) => ValueTypeInfo::Bytes,
            Self::Vec(..) => ValueTypeInfo::Vec,
            Self::Tuple(..) => ValueTypeInfo::Tuple,
            Self::Object(..) => ValueTypeInfo::Object,
            Self::Type(hash) => ValueTypeInfo::Type(*hash),
            Self::Future(..) => ValueTypeInfo::Future,
            Self::Option(..) => ValueTypeInfo::Option,
            Self::Result(..) => ValueTypeInfo::Result,
            Self::TypedObject(typed_object) => {
                let ty = typed_object.get_ref()?.ty;
                ValueTypeInfo::TypedObject(ty)
            }
            Self::TypedTuple(typed_tuple) => {
                let ty = typed_tuple.get_ref()?.ty;
                ValueTypeInfo::TypedTuple(ty)
            }
            Self::External(external) => ValueTypeInfo::External(external.get_ref()?.type_name()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn test_size() {
        // :( - make this 16 bytes again by reducing the size of the Rc.
        assert_eq! {
            std::mem::size_of::<Value>(),
            24,
        };
    }
}
