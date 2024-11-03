use daprs::prelude::*;
use mlua::prelude::*;
use serde::Serialize;

#[derive(Clone, Serialize, FromLua)]
pub struct LuaGraph(pub(crate) Graph);

impl LuaUserData for LuaGraph {}
