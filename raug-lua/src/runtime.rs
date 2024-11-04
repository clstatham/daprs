use mlua::prelude::*;
use raug::prelude::*;
use serde::Serialize;

#[derive(Clone, Serialize, FromLua)]
pub struct LuaRuntime(pub(crate) Runtime);

impl LuaUserData for LuaRuntime {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("run_for", |_, this, duration: f64| {
            this.0
                .run_for(
                    Duration::from_secs_f64(duration),
                    Backend::Default,
                    Device::Default,
                )
                .unwrap();
            Ok(())
        });
    }
}
