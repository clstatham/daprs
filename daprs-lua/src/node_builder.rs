use daprs::builder::static_node_builder::{StaticInput, StaticNode, StaticOutput};
use mlua::prelude::*;
use serde::Serialize;

#[derive(Clone, Serialize, FromLua)]
pub struct LuaNode(pub(crate) StaticNode);

impl LuaUserData for LuaNode {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("input", |_, this, input: u32| {
            Ok(LuaInput(this.0.input(input)))
        });

        methods.add_method("input_named", |_, this, input: String| {
            Ok(LuaInput(this.0.input(input.as_str())))
        });

        methods.add_method("output", |_, this, output: u32| {
            Ok(LuaOutput(this.0.output(output)))
        });

        methods.add_method("output_named", |_, this, output: String| {
            Ok(LuaOutput(this.0.output(output.as_str())))
        });
    }
}

#[derive(Clone, Serialize, FromLua)]
pub struct LuaInput(pub(crate) StaticInput);

impl LuaUserData for LuaInput {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("connect", |_, this, output: LuaOutput| {
            this.0.connect(&output.0);
            Ok(())
        });

        methods.add_method("set", |_, this, value: f64| {
            this.0.set(value);
            Ok(())
        });
    }
}

#[derive(Clone, Serialize, FromLua)]
pub struct LuaOutput(pub(crate) StaticOutput);

impl LuaUserData for LuaOutput {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("connect", |_, this, input: LuaInput| {
            this.0.connect(&input.0);
            Ok(())
        });
    }
}
