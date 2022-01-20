//! Authorization

extern crate serde;
extern crate serde_json;

mod auth_data;
pub mod manager;
pub mod service;

#[doc(inline)]
pub use auth_data::User;

/// Authorization Middlewere
pub struct AuthMiddlewere {}
