# tealr_doc_gen

Tealr_doc_gen is the part of tealr that creates the documentation/definition files. It is split up into several parts to allow people to get only the parts they need.

# Core

This contains the lowest level functions to work with the gathered types, setup the right file structure, etc.

Use this if you just need to be able to generate definition files for lua language server, want to use your own templates that are not dependent on lua/etlua, etc.

# Lib

This contains higher level functions, it comes with the needed templates to produce teal definition files, the site, etc.

Use this if you want to be able to do everything that the binary can.

Reexports core.

# Bin

the source code for the binary. Just a wrapper around the Lib with clap, etc.
