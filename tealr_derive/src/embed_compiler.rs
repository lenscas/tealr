#[cfg(feature = "embed_compiler_from_download")]
pub mod download_compiler;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub mod load_from_disk;

#[cfg(all(
    not(feature = "embed_compiler_from_download"),
    feature = "embed_compiler_from_local"
))]
pub mod download_compiler_mock;

use load_from_disk::discover_tl_tl;
use proc_macro2::Group;
use syn::{parse::Parse, Ident, LitStr};

#[cfg(feature = "embed_compiler_from_download")]
use self::download_compiler::{download_teal_from_github, download_teal_from_luarocks};

#[cfg(all(
    not(feature = "embed_compiler_from_download"),
    feature = "embed_compiler_from_local"
))]
use self::download_compiler_mock::{download_teal_from_github, download_teal_from_luarocks};

use self::load_from_disk::get_local_teal;

#[derive(Debug)]
pub(crate) enum DownloadSource {
    Github,
    Luarocks,
}

pub(crate) enum Source {
    Download(DownloadSource),
    Local,
}
impl From<String> for Source {
    fn from(x: String) -> Self {
        let x = x.to_lowercase();
        let x: &str = &x;
        match x {
            "github" => Source::Download(DownloadSource::Github),
            "luarocks" => Source::Download(DownloadSource::Luarocks),
            "local" => Source::Local,
            x => panic!("Source `{}` is not a supported source.", x),
        }
    }
}

pub(crate) enum EmbedOptions {
    Download {
        source: DownloadSource,
        version: String,
    },
    Local {
        path: String,
    },
}

enum Checker {
    Start,
    V,
    Number,
    Dot,
}

fn get_version(version: String) -> String {
    let mut last = Checker::Start;
    let is_valid_version = version.char_indices().all(|(loc, chara)| {
        if loc == 0 {
            return if chara == 'v' {
                last = Checker::V;
                true
            } else {
                false
            }
        }
        match last {
            Checker::V => {
                if chara.is_ascii_digit() {
                    last = Checker::Number;
                    return true;
                }
                false
            }
            Checker::Number => {
                if chara == '.' {
                    last = Checker::Dot;
                    return true;
                }
                chara.is_ascii_digit()
            }
            Checker::Dot => {
                if chara.is_ascii_digit() {
                    last = Checker::Number;
                    return true;
                }
                false
            }
            Checker::Start => {
                unreachable!()
            }
        }
    });
    if !is_valid_version {
        panic!(
            "{}",
            "Given version is not valid. Versions should look like v{integer}.{integer}.{integer}."
        )
    }
    version
}

pub(crate) struct SourceParameters {
    left: String,
    right: String,
}
impl Parse for SourceParameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left = input.parse::<Ident>()?.to_string();
        input.parse::<syn::Token!(=)>()?;
        let right = input.parse::<LitStr>()?.value();
        Ok(Self { left, right })
    }
}

impl Parse for EmbedOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse::<LitStr>()
            .map(|v| v.value())
            .map(get_version)
            .map(|v| EmbedOptions::Download {
                source: DownloadSource::Github,
                version: v,
            })
            .or_else(|_| {
                let source :Source = input.parse::<Ident>()?.to_string().into();
                let group = input.parse::<Group>()?.stream();
                let assign = if group.is_empty() {
                    None
                } else {
                    Some(syn::parse::<SourceParameters>(group.into())?)
                };
                match source {
                    Source::Download( x) => {
                        let assign = assign.expect("When using a download source, you need to give a version.Example :\nGithub(version=\"v.0.10.0\")");
                        if assign.left.to_lowercase() != "version" {
                            panic!("Invalid parameter. Expected `version`, got `{}`",assign.left);
                        }
                        let version = get_version(assign.right);
                        Ok(EmbedOptions::Download {
                            source : x,
                            version
                        })
                    }
                    Source::Local => {
                        let assign = assign.unwrap_or_else(|| SourceParameters{left : String::from("path"), right : discover_tl_tl()});
                        if assign.left.to_lowercase() != "path"{
                            panic!("Invalid parameter. Expected `path` or empty, got `{}`",assign.left);
                        }
                        Ok(EmbedOptions::Local{path:assign.right})
                    }
                }
            })
    }
}

pub(crate) fn get_teal(source: EmbedOptions) -> String {
    match source {
        EmbedOptions::Download {
            source: DownloadSource::Github,
            version,
        } => download_teal_from_github(version),
        EmbedOptions::Download {
            source: DownloadSource::Luarocks,
            version,
        } => download_teal_from_luarocks(version),
        EmbedOptions::Local { path } => get_local_teal(path),
    }
}
