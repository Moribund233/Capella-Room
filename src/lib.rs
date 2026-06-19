//! Capella Room - Axum WebSocket聊天室应用
//!
//! 这是一个基于Axum、WebSocket和PostgreSQL构建的实时聊天室应用。
//! 支持分布式部署，通过 Redis Pub/Sub 实现跨节点消息广播。

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod oauth;
pub mod redis;
pub mod routes;
pub mod services;
pub mod state;
pub mod test_helpers;
pub mod utils;
pub mod websocket;
