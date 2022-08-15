// configure as no_std as default since it is the default feature
#![cfg_attr(feature = "no_std", no_std)]

pub mod grain;
pub mod manager;
pub mod scheduler;
pub mod source;
pub mod window_function;
