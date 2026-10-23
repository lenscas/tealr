# tealr_doc_gen_lib

Part of [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen), this crate contains everything needed to generate html documentation as well as type definitions.

This makes it easy to make your own program to create the documentation/type definition files, or embed this functionality in the same project that exposes the lua api being documented.

If you just want to take a serialized `TypeWalker` and use it to generate the documentation site + type definition files then you probably should look at [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen) instead

# Lib vs Core

Unlike `tealr_doc_gen_core`, `tealr_doc_gen_lib` comes with a full implementation to generate the definition files and the html documentation. However unlike `tealr_doc_gen_core`, it makes a couple of sacrifices that may not be welcome.

For example, the generation of these files is done through templates written in etlua. This means that doing this fundamentally requires executing lua code.

If this is undesirable, `tealr_doc_gen_core` makes less assumptions and comes with everything needed to work with a `TypeWalker` from `tealr`, at the cost of having to supply your own rendering function to generate the html documentation and the type definitions.
