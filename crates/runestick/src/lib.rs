//! <div align="center">
//! <a href="https://rune-rs.github.io/rune/">
//!     <b>Read the Book 📖</b>
//! </a>
//! </div>
//!
//! <br>
//!
//! <div align="center">
//! <a href="https://github.com/rune-rs/rune/actions">
//!     <img alt="Build Status" src="https://github.com/rune-rs/rune/workflows/Build/badge.svg">
//! </a>
//!
//! <a href="https://github.com/rune-rs/rune/actions">
//!     <img alt="Book Status" src="https://github.com/rune-rs/rune/workflows/Book/badge.svg">
//! </a>
//!
//! <a href="https://crates.io/crates/rune">
//!     <img alt="crates.io" src="https://img.shields.io/crates/v/rune.svg">
//! </a>
//!
//! <a href="https://docs.rs/rune">
//!     <img alt="docs.rs" src="https://docs.rs/rune/badge.svg">
//! </a>
//!
//! <a href="https://discord.gg/v5AeNkT">
//!     <img alt="Chat on Discord" src="https://img.shields.io/discord/558644981137670144.svg?logo=discord&style=flat-square">
//! </a>
//! </div>
//!
//! <br>
//!
//! A stack-based virtual machine for the Rust programming language.
//!
//! This is the driver for the [Rune language].
//! [Rune Language]: https://github.com/rune-rs/rune

#![deny(missing_docs)]

mod any;
mod context;
mod value;
mod vm;
#[macro_use]
mod macros;
mod access;
mod args;
mod assembly;
mod awaited;
mod bytes;
mod call;
mod debug;
mod function;
mod future;
mod generator;
mod generator_state;
mod hash;
mod inst;
mod item;
mod meta;
pub mod module;
pub mod modules;
mod names;
mod panic;
mod protocol;
mod reflection;
mod select;
mod serde;
mod shared;
mod source;
mod span;
mod stack;
mod static_string;
mod static_type;
mod stream;
mod tuple;
mod type_;
mod type_info;
mod unit;
mod vec_tuple;
mod vm_call;
mod vm_error;
mod vm_execution;
mod vm_halt;

impl_external!(anyhow::Error);

/// Exported result type for convenience.
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

/// Exported boxed error type for convenience.
pub type Error = anyhow::Error;

pub use self::args::Args;
pub use self::assembly::{Assembly, Label};
pub use self::generator::Generator;
pub use self::generator_state::GeneratorState;
pub use self::meta::{Meta, MetaClosureCapture, MetaStruct, MetaTuple};
pub use self::module::Module;
pub use self::select::Select;
pub use self::source::Source;
pub use self::span::Span;
pub use self::static_string::StaticString;
pub use self::static_type::{
    StaticType, BOOL_TYPE, BYTES_TYPE, BYTE_TYPE, CHAR_TYPE, FLOAT_TYPE, FUNCTION_TYPE,
    FUTURE_TYPE, GENERATOR_STATE_TYPE, GENERATOR_TYPE, INTEGER_TYPE, OBJECT_TYPE, OPTION_TYPE,
    RESULT_TYPE, STREAM_TYPE, STRING_TYPE, TUPLE_TYPE, UNIT_TYPE, VEC_TYPE,
};
pub use self::stream::Stream;
pub use self::tuple::Tuple;
pub use self::type_::Type;
pub use self::type_info::TypeInfo;
pub use crate::access::{
    AccessError, BorrowMut, BorrowRef, NotAccessibleMut, NotAccessibleRef, RawBorrowedMut,
    RawBorrowedRef,
};
pub use crate::any::Any;
pub use crate::awaited::Awaited;
pub use crate::bytes::Bytes;
pub use crate::call::Call;
pub use crate::context::{Context, ContextError, IntoInstFnHash};
pub use crate::debug::{DebugInfo, DebugInst};
pub use crate::function::Function;
pub use crate::future::Future;
pub use crate::hash::{Hash, IntoHash};
pub use crate::inst::{Inst, PanicReason, TypeCheck};
pub use crate::item::{Component, Item};
pub use crate::names::Names;
pub use crate::panic::Panic;
pub use crate::protocol::{
    Protocol, ADD, ADD_ASSIGN, DIV, DIV_ASSIGN, INDEX_GET, INDEX_SET, INTO_FUTURE, INTO_ITER, MUL,
    MUL_ASSIGN, NEXT, REM, STRING_DISPLAY, SUB, SUB_ASSIGN,
};
pub use crate::reflection::{FromValue, ToValue, UnsafeFromValue, ValueType};
pub use crate::shared::{OwnedMut, OwnedRef, RawOwnedMut, RawOwnedRef, Shared};
pub use crate::stack::{Stack, StackError};
pub use crate::unit::{
    ImportEntry, ImportKey, LinkerError, LinkerErrors, Unit, UnitError, UnitFnKind,
};
pub use crate::value::{
    Integer, Object, TypedObject, TypedTuple, Value, VariantObject, VariantTuple,
};
pub use crate::vec_tuple::VecTuple;
pub use crate::vm::{CallFrame, Vm};
pub use crate::vm_call::VmCall;
pub use crate::vm_error::{VmError, VmErrorKind};
pub use crate::vm_execution::VmExecution;
pub use crate::vm_halt::{VmHalt, VmHaltInfo};

mod collections {
    pub use hashbrown::HashMap;
    pub use hashbrown::HashSet;
}
