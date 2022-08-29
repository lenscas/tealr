use std::borrow::Cow;

use crate::{type_walker::GlobalInstance, KindOfType, TypeName};

pub(crate) struct InstanceWalker {
    doc: String,
    pub(crate) instances: Vec<GlobalInstance>,
}
#[cfg(feature = "mlua")]
impl<'lua> crate::mlu::InstanceCollector<'lua> for InstanceWalker {
    fn add_instance<P, T, F>(&mut self, global_name: P, _: F) -> Result<(), mlua::Error>
    where
        P: Into<Cow<'static, str>>,
        T: TypeName,
        F: FnOnce(&'lua mlua::Lua) -> Result<T, mlua::Error>,
    {
        self.add_instance::<T>(global_name.into());
        Ok(())
    }
    fn document_instance(&mut self, doc: &'static str) {
        self.document_instance(doc)
    }
}

#[cfg(feature = "rlua")]
impl<'lua> crate::rlu::InstanceCollector<'lua> for InstanceWalker {
    fn add_instance<
        P: Into<Cow<'static, str>>,
        T: TypeName,
        F: FnOnce(rlua::Context<'lua>) -> rlua::Result<T>,
    >(
        &mut self,
        global_name: P,
        _: F,
    ) -> Result<(), rlua::Error> {
        self.add_instance::<T>(global_name.into());
        Ok(())
    }
    fn document_instance(&mut self, doc: &'static str) {
        self.document_instance(doc)
    }
}

impl InstanceWalker {
    pub(crate) fn new() -> Self {
        Self {
            doc: Default::default(),
            instances: Default::default(),
        }
    }
    fn add_instance<T: TypeName>(&mut self, name: Cow<'static, str>) {
        let teal_type = T::get_type_parts_as_global();
        let z = T::get_type_kind();
        let is_external = matches!(z, KindOfType::External);
        let doc = std::mem::take(&mut self.doc);
        self.instances.push(GlobalInstance {
            name,
            teal_type,
            is_external,
            doc,
        });
    }
    fn document_instance(&mut self, doc: &'static str) {
        self.doc.push_str(doc);
        self.doc.push('\n');
    }
}

impl Default for InstanceWalker {
    fn default() -> Self {
        Self::new()
    }
}
