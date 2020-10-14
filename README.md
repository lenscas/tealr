# tealr
A wrapper around rlua to help with embedding teal

This crate adds some traits that replace those from RLua, allowing it to generate the .d.tl files needed for teal.
If possible, it will also contain methods that prepare the lua vm so it can load teal files directly.

These 2 things combined should make teal (almost) as easy to embed lua in your rust project. 
