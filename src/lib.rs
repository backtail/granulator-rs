// configure as no_std as default since it is the default feature
#![cfg_attr(feature = "no_std", no_std)]

pub(crate) mod manager;

pub(crate) mod audio_tools;
pub(crate) mod grain;
pub(crate) mod grains_vector;
pub(crate) mod pointer_wrapper;
pub(crate) mod scheduler;
pub(crate) mod source;
pub(crate) mod window_function;

pub use crate::manager::Granulator;
