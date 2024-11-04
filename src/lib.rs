#![doc = include_str!("../README.md")]
#![cfg_attr(doc, warn(missing_docs))]

use cpal::traits::{DeviceTrait, HostTrait};
use runtime::Backend;

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod message;
pub mod processor;
pub mod runtime;
pub mod signal;

/// Re-exports of the most commonly used types and traits.
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::builder::{
        graph_builder::GraphBuilder,
        node_builder::{IntoNode, Node},
        static_graph_builder::StaticGraphBuilder,
        static_node_builder::{IntoStaticNode, StaticNode},
    };
    pub use crate::builtins::*;
    pub use crate::graph::Graph;
    pub use crate::message::*;
    pub use crate::processor::{Process, Processor, ProcessorError, SignalSpec};
    pub use crate::runtime::{Backend, Device, Runtime};
    pub use crate::signal::{Buffer, Sample, Signal, SignalBuffer};
    pub use std::time::Duration;
    pub use typetag;
}

/// Returns a Vec of available backends.
pub fn available_backends() -> Vec<Backend> {
    let mut backends = vec![Backend::Default];
    for host in cpal::available_hosts() {
        match host {
            #[cfg(all(target_os = "linux", feature = "jack"))]
            cpal::HostId::Jack => {
                backends.push(Backend::Jack);
            }
            #[cfg(target_os = "linux")]
            cpal::HostId::Alsa => {
                backends.push(Backend::Alsa);
            }
            #[cfg(target_os = "windows")]
            cpal::HostId::Wasapi => {
                backends.push(Backend::Wasapi);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    backends
}

/// Prints the available backends.
pub fn list_backends() {
    println!("Listing available backends:");
    for (i, backend) in available_backends().into_iter().enumerate() {
        println!("  {}: {:?}", i, backend);
    }
}

/// Prints the available devices for the given backend.
pub fn list_devices(backend: Backend) {
    println!("Listing devices for backend: {:?}", backend);
    let host = match backend {
        Backend::Default => cpal::default_host(),
        #[cfg(all(target_os = "linux", feature = "jack"))]
        Backend::Jack => cpal::host_from_id(cpal::HostId::Jack).unwrap(),
        #[cfg(target_os = "linux")]
        Backend::Alsa => cpal::host_from_id(cpal::HostId::Alsa).unwrap(),
        #[cfg(target_os = "windows")]
        Backend::Wasapi => cpal::host_from_id(cpal::HostId::Wasapi).unwrap(),
    };
    for (i, device) in host.output_devices().unwrap().enumerate() {
        println!("  {}: {:?}", i, device.name());
    }
}
