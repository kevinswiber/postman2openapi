//! Support for OpenApi version 3.0.1 specification.
//!
//! See the
//! [specification](https://github.com/OAI/OpenAPI-Specification/blob/0dd79f6/versions/3.0.1.md)
//! for more information.

mod components;
mod schema;

pub use self::{components::*, schema::*};
