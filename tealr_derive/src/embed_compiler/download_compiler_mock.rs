//this file replaces `download_compiler` file with functions that panic if the `embed_compiler_from_download` is not set

pub(crate) fn download_teal_from_luarocks(_: String) -> String {
    panic!("You can only download if the feature `embed_compiler_from_download` is enabled")
}

pub(crate) fn download_teal_from_github(_: String) -> String {
    panic!("You can only download if the feature `embed_compiler_from_download` is enabled")
}
