#![doc = include_str!("../README.md")]

use cpal::traits::{DeviceTrait, HostTrait};
use runtime::Backend;

pub mod graph;
pub mod processors;
pub mod runtime;
pub mod signal;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::add_node;
    pub use crate::graph::{
        builder::{GraphBuilder, Node},
        edge::Edge,
        node::Process,
        Graph,
    };
    pub use crate::processors::{env::*, functional::*, graph::*, io::*, math::*, osc::*, time::*};
    pub use crate::runtime::{Backend, Device, Runtime};
    pub use crate::signal::{
        Buffer, Sample, Signal, SignalData, SignalKind, SignalRate, SignalSpec,
    };
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    pub fn test_runtime_offline() {
        let graph = GraphBuilder::default();
        let time = graph.processor(Time::ar());
        let two_pi = graph.processor(Constant::ar(std::f64::consts::TAU.into()));
        let freq = graph.processor(Constant::ar(2.0.into()));

        let sine_wave = (time * freq * two_pi).sin();

        let out = graph.output();
        out.connect_inputs([(sine_wave, 0)]);

        let mut runtime = Runtime::new(graph.build().unwrap());

        let bufs = runtime
            .run_offline(std::time::Duration::from_secs(2), 32.0, 32.0, 4)
            .unwrap();
        assert_eq!(bufs.len(), 1);
        let buf = &bufs[0];
        assert_eq!(buf.len(), 64);

        let mut sum = 0.0f64;
        for i in 0..64 {
            sum += *buf[i];
            println!("{}", *buf[i]);
        }
        assert!(sum.abs() < 1e-5);
    }
}
