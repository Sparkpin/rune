//! The `core` package.
//!
//! Contains functions such as:
//! * `dbg` to debug print to stdout.

use crate::context::{ContextError, Module};
use crate::value::ValuePtr;

/// Install the core package into the given functions namespace.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::new(&["std", "result"]);

    module
        .ty(&["Result"])
        .build::<Result<ValuePtr, ValuePtr>>()?;

    module.function(&["Result", "Err"], Result::<ValuePtr, ValuePtr>::Err)?;
    module.function(&["Result", "Ok"], Result::<ValuePtr, ValuePtr>::Ok)?;
    Ok(module)
}