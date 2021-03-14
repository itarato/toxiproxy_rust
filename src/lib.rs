#[macro_use]
extern crate lazy_static;

pub mod client;
pub mod consts;
pub mod http_client;
pub mod proxy;
mod toxic;

use client::*;

lazy_static! {
    pub static ref TOXIPROXY: Client = Client::new("127.0.0.1:8474");
}
