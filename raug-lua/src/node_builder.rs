use raug::{
    builder::static_node_builder::{StaticInput, StaticNode, StaticOutput},
    prelude::Message,
};
use mlua::prelude::*;
use serde::Serialize;

use crate::LuaBang;

#[derive(Clone, Serialize, FromLua)]
pub struct LuaNode(pub(crate) StaticNode);

impl LuaUserData for LuaNode {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("input", |_, this, input: LuaValue| match input {
            LuaValue::Integer(index) => Ok(LuaInput(this.0.input(index as u32))),
            LuaValue::String(name) => Ok(LuaInput(this.0.input(&*name.to_string_lossy()))),
            _ => Err(mlua::Error::external("Invalid input type")),
        });

        methods.add_method("output", |_, this, output: LuaValue| match output {
            LuaValue::Integer(index) => Ok(LuaOutput(this.0.output(index as u32))),
            LuaValue::String(name) => Ok(LuaOutput(this.0.output(&*name.to_string_lossy()))),
            _ => Err(mlua::Error::external("Invalid output type")),
        });

        methods.add_method("m2s", |_, this, _args: ()| Ok(LuaNode(this.0.m2s())));
        methods.add_method("s2m", |_, this, _args: ()| Ok(LuaNode(this.0.s2m())));

        methods.add_meta_method("__add", |_, this, other: LuaValue| match other {
            LuaValue::Number(float) => Ok(LuaNode(&this.0 + float)),
            LuaValue::UserData(data) if data.is::<LuaNode>() => {
                Ok(LuaNode(&this.0 + &data.borrow::<LuaNode>().unwrap().0))
            }
            _ => Err(mlua::Error::external("Invalid operand type")),
        });

        methods.add_meta_method("__sub", |_, this, other: LuaValue| match other {
            LuaValue::Number(float) => Ok(LuaNode(&this.0 - float)),
            LuaValue::UserData(data) if data.is::<LuaNode>() => {
                Ok(LuaNode(&this.0 - &data.borrow::<LuaNode>().unwrap().0))
            }
            _ => Err(mlua::Error::external("Invalid operand type")),
        });

        methods.add_meta_method("__mul", |_, this, other: LuaValue| match other {
            LuaValue::Number(float) => Ok(LuaNode(&this.0 * float)),
            LuaValue::UserData(data) if data.is::<LuaNode>() => {
                Ok(LuaNode(&this.0 * &data.borrow::<LuaNode>().unwrap().0))
            }
            _ => Err(mlua::Error::external("Invalid operand type")),
        });

        methods.add_meta_method("__div", |_, this, other: LuaValue| match other {
            LuaValue::Number(float) => Ok(LuaNode(&this.0 / float)),
            LuaValue::UserData(data) if data.is::<LuaNode>() => {
                Ok(LuaNode(&this.0 / &data.borrow::<LuaNode>().unwrap().0))
            }
            _ => Err(mlua::Error::external("Invalid operand type")),
        });

        methods.add_meta_method("__unm", |_, this, _: ()| Ok(LuaNode(-&this.0)));

        macro_rules! impl_unary_op {
            ($($method:ident),*) => {
                $(
                    methods.add_method(stringify!($method), |_, this, _: ()| {
                        Ok(LuaNode(this.0.$method()))
                    });
                )*
            };
        }

        macro_rules! impl_binary_op {
            ($($method:ident),*) => {
                $(
                    methods.add_method(stringify!($method), |_, this, other: f64| {
                        Ok(LuaNode(this.0.$method(other)))
                    });
                )*
            };
        }

        impl_unary_op!(
            sin, cos, tan, asin, acos, atan, sqrt, cbrt, ceil, floor, round, abs, signum, fract,
            recip
        );

        impl_binary_op!(powf, add, sub, mul, div, rem, min, max, atan2, hypot);
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

        methods.add_method("set", |_, this, value: LuaValue| {
            match value {
                LuaValue::UserData(data) if data.is::<LuaBang>() => this.0.set(Message::Bang),
                LuaValue::Number(float) => this.0.set(float),
                LuaValue::Integer(int) => this.0.set(Message::Int(int)),
                LuaValue::String(string) => this.0.set(Message::String(string.to_string_lossy())),
                _ => return Err(mlua::Error::external("Invalid message type")),
            }
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
