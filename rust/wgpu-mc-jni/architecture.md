# What this is

This is the crate that interfaces with the Electrum mod in Fabric. In simple words, this crate "describes" Minecraft
to wgpu-mc.

# How it works

This crate (and it's dependencies) are bundled in a dylib. It's then called from Electrum.

## Initialization

There's some weird Minecraft-specific behavior that gets handled in Java that this crate doesn't deal with.
The gist of it, however is:

- Mixins that hook into Minecraft's Registry inform this crate about which blocks exist, and their blockstates with their corresponding names.
- Once that's all done, Electrum tells the crate to bake all the blocks. Once this happens, each BlockState has a corresponding
    BlockstateKey, which is a u32 comprised of two u16s, the highest u16 representing the index into which `Block` in wgpu-mc, and the
    lowest u16 representing the actual block state. The crate then gives Electrum all of the generated BlockstateKeys (which are arbitrary and could technically change each launch)
    and through a mixin accessor, we effectively add a new field (an int) into the Minecraft class `BlockState`

## GUIs

Minecraft abstracts it's GUI code enough so that we really only care about a few classes. A lot of state is dealt with in Java though,
to keep things cleaner in Rust. What Rust actually deals with:
- Each "immediate" GL call which has a return type, such as glGenTexture and glTexSubImage2D. These are opaquely asynchronous wherever possible,
    such as is the case with glTexSubImage2D, which defers the actual logic into another thread so as to keep rendering fast.
- Each draw call that *would* have called OpenGL eventually. We intercept this in our mixin called `BufferRendererMixin.java`
- Essentially all OpenGL calls conveniently go through a wrapper class that Mojang wrote called GlStateManager. We overwrite all the methods in that class
    and make them either do nothing, or do some lightweight state management. Otherwise they call native JNI functions in this crate.

## IO

Keyboard & mouse events are received through the winit EventLoop and are passed into a helper thread (the same thread which handles glTexSubImage2D)
which then calls into the JNI to inform Electrum about the events.

## World rendering

wgpu-mc does the heavily lifting here, but there's still a fair bit of surrounding Minecraft-y state that we either handle in Java,
or in Rust, or a mix of both.
In Rust, we have an implementation of Minecraft's serialized `Palette<T>` structure for deserialization in Rust,
as well as an implementation of PackedIntegerArray. These then serve to form an implementation of wgpu-mc's BlockStateProvider.
We use mixins to figure out when the game would have baked a chunk in Java, and we make wgpu-mc do it instead.

## Sound

Handled by Minecraft, we don't have to do anything

## Mod support

TODO