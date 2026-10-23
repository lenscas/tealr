use crate::{
    app::{Config, get_paths},
    create_docs::{run_from_walker, run_template},
    self_doc::{generate_self_def, generate_self_html_doc},
};

mod app;
mod create_docs;
mod credits;
mod self_doc;

fn main() -> anyhow::Result<()> {
    match get_paths()? {
        app::Modes::Credits => {
            credits::show_credits();
        }
        app::Modes::GenerateDocs(x) => {
            //generate_docs::generate_docs(x)
            run_template(*x)?;
        }
        app::Modes::SelfDocTemplate { build_dir } => {
            let walker = generate_self_html_doc()?;
            run_from_walker(
                walker,
                Config {
                    is_global: true,
                    custom_html_runner: None,
                    json: "{}".into(),
                    name: "tealr_doc_gen".into(),
                    root: "".into(),
                    build_dir,
                    template_kind: app::TemplateKind::Builtin,
                    def_config: Default::default(),
                    lua_addon: Some(app::LuaAddon::Create {
                        words: Default::default(),
                        files: Default::default(),
                        settings: Default::default(),
                    }),
                },
            )?;
        }

        app::Modes::GenFile { file, location } => {
            std::fs::write(location, file)?;
        }
        app::Modes::Nothing => (),
        app::Modes::SelfDefTemplate { build_dir } => {
            let walker = generate_self_def()?;
            run_from_walker(
                walker,
                Config {
                    is_global: true,
                    custom_html_runner: None,
                    json: "{}".into(),
                    name: "tealr_doc_gen".into(),
                    root: "".into(),
                    build_dir,
                    template_kind: app::TemplateKind::Builtin,
                    def_config: Default::default(),
                    lua_addon: Some(app::LuaAddon::Create {
                        words: Default::default(),
                        files: Default::default(),
                        settings: Default::default(),
                    }),
                },
            )?;
        }
    }
    Ok(())
}
