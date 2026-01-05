use crate::{type_walker::GlobalInstance, ToTypename};

pub(crate) struct InstanceWalker {
    doc: String,
    pub(crate) instances: Vec<GlobalInstance>,
}
#[cfg(feature = "mlua")]
impl crate::mlu::InstanceCollector for InstanceWalker {
    fn add_instance<P, T, F>(&mut self, name: P, _: F) -> Result<&mut Self, mlua::Error>
    where
        P: Into<String>,
        T: ToTypename,
        F: FnOnce(&mlua::Lua) -> Result<T, mlua::Error>,
    {
        self.add_instance::<T>(name.into());
        Ok(self)
    }
    fn document_instance(&mut self, doc: &'static str) -> &mut Self {
        self.document_instance(doc);
        self
    }
}

impl InstanceWalker {
    pub(crate) fn new() -> Self {
        Self {
            doc: Default::default(),
            instances: Default::default(),
        }
    }
    #[allow(dead_code)]
    fn add_instance<T: ToTypename>(&mut self, name: String) {
        let doc = std::mem::take(&mut self.doc);
        self.instances.push(GlobalInstance {
            name,
            doc,
            ty: T::to_typename(),
        });
    }
    #[allow(dead_code)]
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
