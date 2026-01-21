//! 中间件模块

pub mod auth;

pub use auth::{AuthenticatedUser, get_authenticated_user};
