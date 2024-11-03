use daprs::prelude::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{graph::LuaGraph, node_builder::LuaNode, runtime::LuaRuntime};

#[derive(Clone, Default, Serialize, Deserialize, FromLua)]
pub struct LuaGraphBuilder(StaticGraphBuilder);

impl LuaUserData for LuaGraphBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("input", |_lua, this, _args: ()| Ok(this.input()));
        methods.add_method("output", |_lua, this, _args: ()| Ok(this.output()));

        methods.add_method("sine_osc", |_lua, this, _args: ()| {
            Ok(LuaNode(this.0.sine_osc()))
        });

        methods.add_method("saw_osc", |_lua, this, _args: ()| {
            Ok(LuaNode(this.0.saw_osc()))
        });

        methods.add_method_mut("build", |_lua, this, _args: ()| {
            Ok(std::mem::take(this).build())
        });

        methods.add_method_mut("build_runtime", |_lua, this, _args: ()| {
            Ok(std::mem::take(this).build_runtime())
        });
    }
}

impl LuaGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> LuaGraph {
        LuaGraph(self.0.build())
    }

    pub fn build_runtime(self) -> LuaRuntime {
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
