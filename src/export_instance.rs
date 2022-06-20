use std::borrow::Cow;

use crate::{KindOfType, NamePart, TypeName};

pub(crate) struct InstanceWalker {
    pub(crate) instances: Vec<(Cow<'static, str>, Cow<'static, [NamePart]>, bool)>,
}
#[cfg(feature = "mlua")]
impl<'lua> crate::mlu::InstanceCollector<'lua> for InstanceWalker {
    fn add_instance<T: TypeName, F: Fn(&'lua mlua::Lua) -> Result<T, mlua::Error>>(
        &mut self,
        global_name: Cow<'static, str>,
        _: F,
    ) -> Result<(), mlua::Error> {
        self.add_instance::<T>(global_name);
        Ok(())
    }
}

#[cfg(feature = "rlua")]
impl<'lua> crate::rlu::InstanceCollector<'lua> for InstanceWalker {
    fn add_instance<T: TypeName, F: Fn(rlua::Context<'lua>) -> rlua::Result<T>>(
        &mut self,
        global_name: Cow<'static, str>,
        _: F,
    ) -> Result<(), rlua::Error> {
        self.add_instance::<T>(global_name);
        Ok(())
    }
}

impl InstanceWalker {
    pub(crate) fn new() -> Self {
        Self {
            instances: Default::default(),
        }
    }
    fn add_instance<T: TypeName>(&mut self, global_name: Cow<'static, str>) {
        let type_name = T::get_type_parts();
        let z = T::get_type_kind();
        let is_external = matches!(z, KindOfType::External);
        self.instances.push((global_name, type_name, is_external));
    }
}

impl Default for InstanceWalker {
    fn default() -> Self {
        Self::new()
    }
}
