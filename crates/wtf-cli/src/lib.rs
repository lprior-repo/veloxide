//! wtf-cli — `wtf serve`, `wtf lint`, `wtf admin rebuild-views`.
//! Implemented in wtf-4mym, wtf-qz46, wtf-creq beads.

pub mod lint;

#[path = "commands/admin.rs"]
pub mod admin;

#[path = "commands/serve.rs"]
pub mod serve;
