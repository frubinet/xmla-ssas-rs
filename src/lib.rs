//! Native Rust client for Microsoft SQL Server Analysis Services
//! using XMLA over TCP/DIME.

#![forbid(unsafe_code)]

mod dime;
mod auth;
pub mod xmla;
pub mod connection;
