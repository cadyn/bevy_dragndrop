## About
This is a simple crate that I've made because I realized the lack of existing crates to implement 
simple drag and drop functionality for bevy. This crate supports drag and drop for both UI and 2d 
world objects with options for modifiers and other mouse buttons. Contributions and issues for bug
reports or feature requests are welcome.

## Usage
Usage is designed to be simple and leave most of the control in the hands of you. 
The main components you'll need are `bevy_dragndrop::Draggable` and `bevy_dragndrop::Receiver`
These components can be attached to any entity with at minimum a transform and GlobalTransform.
They are also compatible with NodeBundles.

Once you have entities with these components, you will be able to make use of the four events
that the library provides to actually provide functionality based on the dragging and dropping.
The four events include `Dropped`, `Dragged`, `HoverChanged`, and `DragAwait`

See the examples for detailed usage, as well as the docs at https://docs.rs/bevy_dragndrop/0.1.0/bevy_dragndrop/

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.