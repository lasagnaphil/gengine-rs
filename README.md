# gengine-rs
An (unfinished) small 2D game framework created in Rust. 

Currently it implements the following:

- Basic windowing (using SDL)
- Resource management (using the Storage struct)
- Resource serialization (using the Storage struct)
- Rendering sprites and tilemaps (using OpenGL)
- Input management

It still doesn't provide

- Any kind of entity system
- Sound
- Networking
- Anything else you will expect in a finished 2D game framework

Nowadays, I've figured out Rust doesn't give that many advantages compared to C/C++ for creating a game / game engine, so this project is currently abandoned.
However, if you want to get some ideas on how to make a simple 2D game framework in Rust wrapping SDL and OpenGL, this might be pretty useful...

The meat of the source code would be ``storage.rs``, which provides a container to store any kind of resource. (However, I haven't tried optimizing it, so modify it as your own liking...)
It use generational indices for providing a reference to a resource, instead of raw/smart pointers. This is widely used in many game engines regardless of implementation language (And has a nice property of being able to circumvent the Rust borrow checker without expensive Rc<T> stuff)

# License

DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE

Version 2, December 2004

Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>

Everyone is permitted to copy and distribute verbatim or modified
copies of this license document, and changing it is allowed as long
as the name is changed.

DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

0. You just DO WHAT THE FUCK YOU WANT TO.
