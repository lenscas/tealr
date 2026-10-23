use anyhow::Context;
use tealr::TypeWalker;
use tealr_doc_gen_lib::definition_files::ShareWithLua;

use crate::self_doc::shared_types;

pub fn generate_self_def() -> Result<TypeWalker, anyhow::Error> {
    let x = tealr::TypeWalker::new();
    shared_types(x)
        .document_global_instance::<ShareWithLua>()
        .context("failed to generate global instance")
}
