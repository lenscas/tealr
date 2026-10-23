mod self_definition_docs;
mod self_html_doc;

use tealr::{
    TypeWalker,
    mlu::{self, UserDataProxy},
};
use tealr_doc_gen_lib::RenderOptions;

pub use self_definition_docs::generate_self_def;
pub use self_html_doc::generate_self_html_doc;

fn shared_types(walker: TypeWalker) -> TypeWalker {
    walker
        .process_type::<RenderOptions<mlu::generics::X>>()
        .process_type::<tealr::FunctionParam>()
        .process_type::<tealr::FunctionRepresentation>()
        .process_type::<tealr::MapRepresentation>()
        .process_type::<tealr::SingleType>()
        .process_type::<tealr::Name>()
        .process_type::<tealr::ExtraPage>()
        .process_type::<tealr::Type>()
        .process_type::<tealr::MacroExp>()
        .process_type::<UserDataProxy<tealr::Type>>()
        .process_type::<tealr::EnumGenerator>()
        .process_type::<tealr::ExportedFunction>()
        .process_type::<tealr::Field>()
        .process_type::<tealr::GlobalInstance>()
        .process_type::<tealr::KindOfType>()
        .process_type::<tealr::NamePart>()
        .process_type::<tealr::RecordGenerator>()
        .process_type::<tealr::TealType>()
        .process_type::<tealr::TypeGenerator>()
        .process_type::<tealr::TypeWalker>()
}
