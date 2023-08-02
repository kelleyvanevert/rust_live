# Rust audio livecode editor

**My slow path towards an audio livecoding system, built entirely in Rust 🦀**

Instead of going there the most pragmatic way (i.e. making a web app, using web audio, etc.), I'm taking the slow path, because I want to play with (/ learn about) all these cool things:

- Rust 🦀 — _because everything is more fun with Rust_ :)
- Window management with [winit](https://github.com/rust-windowing/winit)
  - (I hand-patched it locally to overcome the drag+drop cursor positioning [issue](https://github.com/rust-windowing/winit/issues/1550) with this [proposed PR](https://github.com/rust-windowing/winit/pull/2615).)
- Low-level graphics APIs 🌈 with [wgpu](https://wgpu.rs/)
  - ..confronting me with such things as _bind group layouts_, _diffuse maps_, or putting a _projection mapping_ in a _uniform buffer_, to gain an understanding of low-level graphics APIs such as Vulkan, Metal, etc., that are usually mostly used for building game engines
  - and of course, this way it can be _super fast_ ⚡️
- DSP
  - ..much to learn
- Code editor logic
  - Building the logic around editing code with multiple selections, etc., is quite challenging but fun on its own :P I decided that I might as well make this deep-dive, after I already touched on it with another hobby-project of mine: [ASCII recipes](https://asciirecip.es/)

![](pics/stuff_that_looks_like_live_code.png)

**TODO list**

- Editor logic

  - [ ] double click selects word

- Widgets

  - [ ] Refactor rendering pipeline / state

- Language

  - [ ] Syntax concepting, parsing, highlighting

- DSP
  - [ ] Generate a sine wave
  - [ ] Play a sample
  - ...and so much more
