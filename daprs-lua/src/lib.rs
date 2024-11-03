use mlua::prelude::*;

use graph_builder::graph_builder;
use serde::Serialize;

pub mod graph;
pub mod graph_builder;
pub mod node_builder;
pub mod runtime;

#[derive(Clone, Serialize, FromLua)]
pub struct LuaBang;

impl LuaUserData for LuaBang {}

pub fn bang(_: &Lua, _: ()) -> LuaResult<LuaBang> {
    Ok(LuaBang)
}

#[mlua::lua_module]
fn daprs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("bang", lua.create_function(bang)?)?;
    exports.set("graph_builder", lua.create_function(graph_builder)?)?;
    Ok(exports)
}
