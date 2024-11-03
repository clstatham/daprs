use mlua::prelude::*;

use graph_builder::graph_builder;

pub mod graph;
pub mod graph_builder;
pub mod node_builder;
pub mod runtime;

#[mlua::lua_module]
fn daprs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("graph_builder", lua.create_function(graph_builder)?)?;
    Ok(exports)
}
