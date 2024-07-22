#![allow(clippy::new_without_default)]

use std::sync::{Arc, Mutex};

use pyo3::{prelude::*, types::PyDict};

#[pyclass]
pub struct Node {
    pub node: papr::prelude::Node,
}

#[pymethods]
impl Node {
    fn __add__(&self, other: &Self) -> Self {
        Node {
            node: self.node.clone() + other.node.clone(),
        }
    }

    fn __mul__(&self, other: &Self) -> Self {
        Node {
            node: self.node.clone() * other.node.clone(),
        }
    }

    fn __sub__(&self, other: &Self) -> Self {
        Node {
            node: self.node.clone() - other.node.clone(),
        }
    }

    fn __div__(&self, other: &Self) -> Self {
        Node {
            node: self.node.clone() / other.node.clone(),
        }
    }

    fn __neg__(&self) -> Self {
        Node {
            node: -self.node.clone(),
        }
    }

    fn __pos__(&self) -> Self {
        Node {
            node: self.node.clone(),
        }
    }
}

#[pyclass]
pub struct GraphBuilder {
    pub graph: papr::prelude::GraphBuilder,
}

#[pymethods]
impl GraphBuilder {
    #[new]
    #[pyo3(signature = (graph = None))]
    pub fn new(graph: Option<Graph>) -> Self {
        if let Some(graph) = graph {
            GraphBuilder {
                graph: papr::prelude::GraphBuilder::from_graph(graph.graph),
            }
        } else {
            GraphBuilder {
                graph: papr::prelude::GraphBuilder::new(),
            }
        }
    }

    pub fn input(&self) -> Node {
        Node {
            node: self.graph.input(),
        }
    }

    pub fn output(&self) -> Node {
        Node {
            node: self.graph.output(),
        }
    }

    pub fn kr_constant(&self, value: f64) -> Node {
        Node {
            node: self.graph.kr_constant(value),
        }
    }

    pub fn ar_constant(&self, value: f64) -> Node {
        Node {
            node: self.graph.ar_constant(value),
        }
    }

    pub fn build(&self) -> Graph {
        Graph {
            graph: self.graph.build(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Graph {
    pub graph: papr::prelude::Graph,
}

#[pymethods]
impl Graph {
    #[new]
    pub fn new() -> Self {
        Graph {
            graph: papr::prelude::Graph::default(),
        }
    }

    pub fn into_builder(&self) -> GraphBuilder {
        GraphBuilder {
            graph: self.graph.clone().builder(),
        }
    }
}

#[pyclass]
pub struct Runtime {
    runtime: Arc<Mutex<Option<papr::prelude::Runtime>>>,
}

#[pymethods]
impl Runtime {
    #[new]
    pub fn new(graph: Graph) -> Self {
        Runtime {
            runtime: Arc::new(Mutex::new(Some(papr::prelude::Runtime::new(graph.graph)))),
        }
    }

    #[pyo3(signature = (**kwargs))]
    pub fn run(&self, kwargs: Option<&Bound<'_, PyDict>>) {
        let (backend, device, control_rate) = match kwargs {
            Some(kwargs) => {
                let backend = kwargs
                    .get_item("backend")
                    .map(|item| item.unwrap().extract::<String>().unwrap())
                    .unwrap_or("default".to_string());
                let device = kwargs
                    .get_item("device")
                    .map(|item| item.unwrap().extract::<String>().unwrap())
                    .unwrap_or("default".to_string());
                let control_rate = kwargs
                    .get_item("control_rate")
                    .map(|item| item.unwrap().extract::<f64>().unwrap())
                    .unwrap_or(480.0);
                (backend, device, control_rate)
            }
            None => ("default".to_string(), "default".to_string(), 480.0),
        };
        let backend = match backend.as_str() {
            "default" => papr::prelude::Backend::Default,
            #[cfg(target_os = "linux")]
            "jack" => papr::prelude::Backend::Jack,
            #[cfg(target_os = "linux")]
            "alsa" => papr::prelude::Backend::Alsa,
            #[cfg(target_os = "windows")]
            "wasapi" => papr::prelude::Backend::Wasapi,
            _ => panic!("Unknown backend: {}", backend),
        };
        let device = match device.as_str() {
            "default" => papr::prelude::Device::Default,
            _ => papr::prelude::Device::Name(device.to_string()),
        };
        self.runtime
            .lock()
            .unwrap()
            .take()
            .expect("Runtime already running")
            .run(backend, device, control_rate);
    }
}

/// Python bindings for the `PAPR` audio processing graph library.
#[pymodule(name = "papr")]
fn papr_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Node>()?;
    m.add_class::<GraphBuilder>()?;
    m.add_class::<Graph>()?;
    m.add_class::<Runtime>()?;
    Ok(())
}
