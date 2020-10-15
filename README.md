# tealr
A wrapper around rlua to help with embedding teal

# WARNING!
This is a VERY EARLY release. MANY rust types do NOT yet have the correct traits implemented.

This means that A LOT of types are NOT yet useable.

It is ***only*** available on crates.io so I can more easily get feedback on the documentation and its API.

#

tealr adds some traits that replace/extend those from RLua, allowing it to generate the `.d.tl` files needed for teal.
If possible, it will also contain methods that prepare the lua vm so it can load teal files directly.

These 2 things combined should make teal (almost) as easy to embed lua in your rust project. 
