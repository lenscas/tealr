use anyhow::Context;
use tealr::TypeWalker;
use tealr_doc_gen_lib::{
    core::{
        configuration::{DefTemplateConfig, DefTemplateKind},
        find_users::{NameAndSignature, User},
        html::{
            CustomPage, IndexPage,
            configuration::{TypeDesc, TypeOrPage},
            sidebar::{Member, SideBar},
        },
    },
    html::SharedWithLua,
};

use crate::self_doc::shared_types;

pub fn generate_self_html_doc() -> anyhow::Result<TypeWalker> {
    use tealr_doc_gen_lib::html::markdown::*;
    shared_types(tealr::TypeWalker::new())
        .process_type::<NameAndSignature>()
        .process_type::<Member>()
        .process_type::<SideBar>()
        .process_type::<DefTemplateConfig>()
        .process_type::<DefTemplateKind>()
        .process_type::<MarkdownEvent>()
        .process_type::<MarkdownEventCreator>()
        .process_type::<MarkdownCodeBlockKind>()
        .process_type::<MarkdownCodeBlockKindCreator>()
        .process_type::<MarkdownHeadingLevel>()
        .process_type::<MarkdownLinkType>()
        .process_type::<MarkdownTag>()
        .process_type::<MarkdownTagCreator>()
        .process_type::<MarkdownAlignment>()
        .process_type::<MarkdownMetadataBlockKind>()
        .process_type::<MarkdownTagEnd>()
        .process_type::<MarkdownBlockQuoteKind>()
        .process_type::<MarkdownAttribute>()
        .process_type::<MarkdownTagEndCreator>()
        .process_type::<User>()
        .process_type::<IndexPage>()
        .process_type::<TypeDesc>()
        .process_type::<CustomPage>()
        .process_type::<TypeOrPage>()
        .document_global_instance::<SharedWithLua>()
        .context("failed to generate global instance")
}
