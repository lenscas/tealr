# Tealr_doc_gen_core

This library is used by [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen). It contains helper functions to consume the `TypeWalker` and emit a documentation site + type definition files. As well as various functions to look for common mistakes (missing types, duplicate types, etc).

This crate on its own however is not able to generate the type definition/documentation site. For this, you have to supply your own rendering function. If you instead want to have fully working implementations for this then look at [tealr_doc_gen_lib](https://crates.io/crates/tealr_doc_gen_lib). That crate contains higher level functions, including functions to generate these files complete with working templates.
