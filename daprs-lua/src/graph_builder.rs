use daprs::prelude::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{graph::LuaGraph, node_builder::LuaNode, runtime::LuaRuntime, LuaBang};

#[derive(Clone, Default, Serialize, Deserialize, FromLua)]
pub struct LuaGraphBuilder(StaticGraphBuilder);

impl LuaUserData for LuaGraphBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("input", |_lua, this, _args: ()| Ok(this.input()));
        methods.add_method("output", |_lua, this, _args: ()| Ok(this.output()));

        methods.add_method(
            "print",
            |_lua, this, (name, value): (Option<LuaValue>, Option<LuaValue>)| {
                let name = name.and_then(|name| name.to_string().ok());
                let value = value.and_then(|value| value.to_string().ok());
                match (name, value) {
                    (Some(name), Some(value)) => {
                        Ok(LuaNode(this.0.print(name.as_str(), value.as_str())))
                    }
                    (Some(name), None) => Ok(LuaNode(this.0.print(name.as_str(), None))),
                    (None, Some(value)) => Ok(LuaNode(this.0.print("", value.as_str()))),
                    (None, None) => Ok(LuaNode(this.0.print(None, None))),
                }
            },
        );

        methods.add_method("constant", |_lua, this, value: f64| {
            Ok(LuaNode(this.0.constant(value)))
        });

        methods.add_method("message", |_lua, this, value: LuaValue| match value {
            LuaValue::UserData(value) if value.is::<LuaBang>() => {
                Ok(LuaNode(this.0.message(Message::Bang)))
            }
            LuaValue::Number(float) => Ok(LuaNode(this.0.message(Message::Float(float)))),
            LuaValue::Integer(int) => Ok(LuaNode(this.0.message(Message::Int(int)))),
            LuaValue::String(string) => Ok(LuaNode(
                this.0.message(Message::String(string.to_string_lossy())),
            )),
            _ => Err(mlua::Error::external("Invalid message type")),
        });

        methods.add_method(
            "constant_message",
            |_lua, this, value: LuaValue| match value {
                LuaValue::UserData(value) if value.is::<LuaBang>() => {
                    Ok(LuaNode(this.0.constant_message(Message::Bang)))
                }
                LuaValue::Number(float) => {
                    Ok(LuaNode(this.0.constant_message(Message::Float(float))))
                }
                LuaValue::Integer(int) => Ok(LuaNode(this.0.constant_message(Message::Int(int)))),
                LuaValue::String(string) => Ok(LuaNode(
                    this.0
                        .constant_message(Message::String(string.to_string_lossy())),
                )),
                _ => Err(mlua::Error::external("Invalid message type")),
            },
        );

        methods.add_method("sine_osc", |_lua, this, _args: ()| {
            Ok(LuaNode(this.0.sine_osc()))
        });

        methods.add_method("saw_osc", |_lua, this, _args: ()| {
            Ok(LuaNode(this.0.saw_osc()))
        });

        methods.add_method("build", |_lua, this, _args: ()| Ok(this.build()));

        methods.add_method("build_runtime", |_lua, this, _args: ()| {
            Ok(this.build_runtime())
        });
    }
}

impl LuaGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> LuaGraph {
        LuaGraph(self.0.build())
    }

    pub fn build_runtime(&self) -> LuaRuntime {
        LuaRuntime(self.0.build_runtime())
    }

    pub fn input(&self) -> LuaNode {
        LuaNode(self.0.input())
    }

    pub fn output(&self) -> LuaNode {
        LuaNode(self.0.output())
    }
}

pub fn graph_builder(lua: &Lua, _args: ()) -> LuaResult<LuaAnyUserData> {
    lua.create_ser_userdata(LuaGraphBuilder::new())
}
