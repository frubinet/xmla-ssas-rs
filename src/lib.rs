// SPDX-License-Identifier: MPL-2.0

//! Native Rust client for Microsoft SQL Server Analysis Services
//! using XMLA over TCP/DIME.

#![forbid(unsafe_code)]

mod auth;
pub mod connection;
mod dime;
pub mod xmla;
