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

#[allow(unused_imports)]
pub mod prelude {
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
    pub use std::time::Duration;
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
