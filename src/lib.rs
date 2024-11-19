#![doc = include_str!("../README.md")]
#![cfg_attr(doc, warn(missing_docs))]
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::excessive_precision)]

use cpal::traits::{DeviceTrait, HostTrait};
use runtime::AudioBackend;

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod processor;
pub mod runtime;
pub mod signal;

/// Re-exports of commonly used types and traits from the crate.
#[allow(unused_imports)]
pub mod prelude {
    pub(crate) use crate as raug;
    pub use crate::builder::{
        graph_builder::GraphBuilder,
        node_builder::{Input, IntoNode, Node, Output},
    };
    pub use crate::builtins::*;
    pub use crate::graph::Graph;
    pub use crate::processor::{
        Processor, ProcessorError, ProcessorInputs, ProcessorOutputs, SignalSpec,
    };
    pub use crate::runtime::{AudioBackend, AudioDevice, MidiPort, Runtime, RuntimeHandle};
    pub use crate::signal::{
        AnySignal, Buffer, Float, List, MidiMessage, Signal, SignalBuffer, SignalType,
    };
    pub use raug_macros::{iter_proc_io_as, split_outputs};
    pub use std::time::Duration;
}

#[doc(hidden)]
mod graph_serde {
    #[cfg(feature = "serde")]
    pub trait GraphSerde: erased_serde::Serialize {}
    #[cfg(feature = "serde")]
    impl<T: ?Sized> GraphSerde for T where T: erased_serde::Serialize {}

    #[cfg(not(feature = "serde"))]
    pub trait GraphSerde {}
    #[cfg(not(feature = "serde"))]
    impl<T: ?Sized> GraphSerde for T {}
}

pub(crate) use graph_serde::GraphSerde;

#[doc(hidden)]
mod logging {
    use std::{
        collections::HashSet,
        sync::{LazyLock, Mutex},
    };

    pub(crate) static LOGGED: LazyLock<Mutex<HashSet<String>>> =
        LazyLock::new(|| Mutex::new(HashSet::with_capacity(16)));

    #[macro_export]
    macro_rules! log_once {
        ($val:expr => error $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Error && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::error!($($msg)*);
            }
        }};
        ($val:expr => warn $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Warn && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::warn!($($msg)*);
            }
        }};
        ($val:expr => info $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Info && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::info!($($msg)*);
            }
        }};
        ($val:expr => debug $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Debug && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::debug!($($msg)*);
            }
        }};
        ($val:expr => trace $($msg:tt)*) => {{
            if log::max_level() >= log::LevelFilter::Trace && $crate::logging::LOGGED.lock().unwrap().insert($val.to_string()) {
                log::trace!($($msg)*);
            }
        }};
    }

    #[macro_export]
    macro_rules! error_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => error $($msg)*);
        };
    }

    #[macro_export]
    macro_rules! warn_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => warn $($msg)*);
        };
    }

    #[macro_export]
    macro_rules! info_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => info $($msg)*);
        };
    }

    #[macro_export]
    macro_rules! debug_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => debug $($msg)*);
        };
    }

    #[macro_export]
    macro_rules! trace_once {
        ($val:expr => $($msg:tt)*) => {
            $crate::log_once!($val => trace $($msg)*);
        };
    }
}

#[doc(hidden)]
#[allow(unused)]
pub mod __itertools {
    pub use itertools::{cons_tuples, izip};
}

/// Returns a list of available audio backends, as exposed by the `cpal` crate.
pub fn available_audio_backends() -> Vec<AudioBackend> {
    let mut backends = vec![];
    for host in cpal::available_hosts() {
        match host {
            #[cfg(all(target_os = "linux", feature = "jack"))]
            cpal::HostId::Jack => {
                backends.push(AudioBackend::Jack);
            }
            #[cfg(target_os = "linux")]
            cpal::HostId::Alsa => {
                backends.push(AudioBackend::Alsa);
            }
            #[cfg(target_os = "windows")]
            cpal::HostId::Wasapi => {
                backends.push(AudioBackend::Wasapi);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    backends
}

/// Prints a list of available audio backends to the console.
pub fn list_audio_backends() {
    println!("Listing available backends:");
    for (i, backend) in available_audio_backends().into_iter().enumerate() {
        println!("  {}: {:?}", i, backend);
    }
}

/// Prints a list of available audio devices for the given backend to the console.
pub fn list_audio_devices(backend: AudioBackend) {
    println!("Listing devices for backend: {:?}", backend);
    let host = match backend {
        AudioBackend::Default => cpal::default_host(),
        #[cfg(all(target_os = "linux", feature = "jack"))]
        AudioBackend::Jack => cpal::host_from_id(cpal::HostId::Jack).unwrap(),
        #[cfg(target_os = "linux")]
        AudioBackend::Alsa => cpal::host_from_id(cpal::HostId::Alsa).unwrap(),
        #[cfg(target_os = "windows")]
        AudioBackend::Wasapi => cpal::host_from_id(cpal::HostId::Wasapi).unwrap(),
    };
    for (i, device) in host.output_devices().unwrap().enumerate() {
        println!("  {}: {:?}", i, device.name());
    }
}

/// Prints a list of available MIDI ports to the console.
pub fn list_midi_ports() {
    let input = midir::MidiInput::new("raug").unwrap();
    println!("Listing available MIDI ports:");
    for (i, port) in input.ports().iter().enumerate() {
        println!(
            "  {}: {:?}",
            i,
            input
                .port_name(port)
                .unwrap_or_else(|_| "Unknown".to_string())
        );
    }
}
