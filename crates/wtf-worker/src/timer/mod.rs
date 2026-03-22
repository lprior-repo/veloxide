pub mod record;
pub mod ops;
pub mod r#loop;

#[cfg(test)]
mod tests;

pub use record::TimerRecord;
pub use ops::{delete_timer, fire_timer, store_timer};
pub use r#loop::{run_timer_loop, TIMER_POLL_INTERVAL};
