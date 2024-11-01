#![doc = include_str!("../README.md")]

use cpal::traits::{DeviceTrait, HostTrait};
use runtime::Backend;

pub mod builder;
pub mod builtins;
pub mod graph;
pub mod message;
pub mod processor;
pub mod runtime;
pub mod signal;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::builder::{
        graph_builder::GraphBuilder,
        node_builder::{IntoNode, Node},
    };
    pub use crate::builtins::*;
    pub use crate::graph::{edge::Edge, Graph};
    pub use crate::message::*;
    pub use crate::processor::{Process, Processor, SignalSpec};
    pub use crate::runtime::{Backend, Device, Runtime};
    pub use crate::signal::{Buffer, Sample};
}

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

pub fn default_backend() -> Backend {
    Backend::Default
}

pub fn list_backends() {
    println!("Listing available backends:");
    for (i, backend) in available_backends().into_iter().enumerate() {
        println!("  {}: {:?}", i, backend);
    }
}

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
