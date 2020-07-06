# Envolvigo – A Transient Designer plugin

Envolvigo is transient designer plugin in LV2 format. It is in an early
development stage. The underlying framework
[rust-lv2](https://github.com/RustAudio/rust-lv2), especially regarding UI is
still under heavy development.

So using and updating this plugin is a little bit bumpy. It is being developed
on Linux, up to now this is the only supported platform.

## Installation

As mentioned installing and updating this will be a little bumpy. Nevertheless
I am trying hard to make it as easy as possible for interested testers or
collaborators.

### Prerequisites

*Envolvigo* is written in Rust. So you need a Rust environment for it.  On
Ubuntu you can install the packages `rustc` and `cargo`. Additionally need to
install `clang`.  On other distros there are probably similar packages. Also
take a look at the recommendations on the [Rust
page](https://www.rust-lang.org/tools/install) and in the [Cargo
Book](https://doc.rust-lang.org/cargo/getting-started/installation.html).


Once you have a running Rust/Cargo setup, clone this repository, and run
```
install_lv2.sh
```
from within the directory from a terminal. You should see a bunch of messages
in your terminal. Finally it should say `envolvigo.lv2 successfully installed`.

Then you should find `Envolvigo` in plugins hosts like Ardour and Carla. There
is a mono and a stereo version available. The uris are

* `http://johannes-mueller.org/lv2/envolvigo\#mono`
* `http://johannes-mueller.org/lv2/envolvigo\#stereo`

This works at least on Linux. About other systems I don't know.


Alternatively you can symlink `lv2-debug` or to
`$HOME/.lv2/envolvigo.lv2`. Then changes you make to `envolvigo` are available
right after `cargo build`.


## Screenshot

![screenshot](https://raw.github.com/johannes-mueller/envolvigo/master/img/envolvigo-screenshot.png "Envolvigo GUI")
