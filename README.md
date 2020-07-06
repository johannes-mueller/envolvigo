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


## Usage

*Envolvigo* tries to automatically detect the transition from the attack regime
to the sustain regime, i.e. the attack part, and the transition from sustain to
release, i.e. the sustain part. Both parts of a beat can independently be
boosted or attenuated by the "boost" parameters. The two "smooth" knobs
smoothen the envelope detectors that detect the transition points. As a rule of
thumb, the smoother the envelopes the later the detected transition
points. They are called smooth, because they also smoothen the character of the
boost and attenuation.

The "Output level" knob selects the level of the output signal *before* it is
mixed with the input signal according to the "Dry/Wet" knob.


## Principle

The detection uses for both parts two envelope detectors, a fast one and a slow
one. The fast one follows the signal level upwards instantly and slowly
releases when the signal level decreases. The slow one vice versa, it follows
the signal level upwards slowly. Thus the fast envelope detector is always at a
higher level as the slow one. The higher the difference the higher the boost or
attenuation.
