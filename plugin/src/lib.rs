use std::f32::consts::PI;

use itertools::izip;
use lv2::prelude::*;
use lv2::lv2_urid as lv2_urid;

#[derive(PortCollection)]
struct Ports {
    enabled: InputPort<Control>,
    use_sidechain: InputPort<Control>,
    attack_boost: InputPort<Control>,
    attack_smooth: InputPort<Control>,
    sustain_boost: InputPort<Control>,
    sustain_smooth: InputPort<Control>,
    _gain_attack: InputPort<Control>,
    _gain_release: InputPort<Control>,
    outgain: InputPort<Control>,
    mix: InputPort<Control>,
    control: InputPort<AtomPort>,
    notify: OutputPort<AtomPort>,
    input: InputPort<Audio>,
    sidechain_input: InputPort<Audio>,
    output: OutputPort<Audio>
}

#[derive(FeatureCollection)]
struct Features<'a> {
    map: LV2Map<'a>,
    options: lv2_urid::LV2Options
}

struct Dezipper {
    target: f32,
    current_value: f32,
    coeff: f32,
}

impl Dezipper {
    fn new(start_value: f32, sample_rate: f32) -> Self {
        Dezipper {
            target: start_value,
            current_value: start_value,
            coeff: 1.0 - (-2.0 * PI * 25. / sample_rate).exp()
        }
    }

    fn set_value(&mut self, v: f32) {
        self.target = v
    }

    fn process(&mut self) -> f32 {
        self.current_value += self.coeff * (self.target - self.current_value);
        self.current_value
    }
}


struct EnvelopeDetector {
    attack: f32,
    release: f32,
    sample_rate: f32,

    current_level: f32,
    y1: f32
}

impl EnvelopeDetector {
    fn new(sample_rate: f32) -> Self {
        EnvelopeDetector {
            attack: 0.0,
            release: 0.0,
            sample_rate,
            current_level: 0.0,
            y1: 0.0
        }
    }

    fn process(&mut self, level: f32) -> f32 {
        if level >= self.current_level {
            self.current_level = self.attack * (self.current_level - level) + level;
        } else {
            self.current_level = self.release * (self.current_level - level) + level;
        }
        self.current_level
    }

    fn level(&self) -> f32 {
        self.current_level
    }

    fn set_params(&mut self, attack_time: f32, release_time: f32) {
        self.attack = (-1.0 / (self.sample_rate * attack_time)).exp();
//        self.y1 = self.attack;
        self.release = (-1.0 / (self.sample_rate * release_time)).exp();
    }

    fn reset(&mut self, level: f32) {
        self.y1 = self.attack;
        self.current_level = level;
    }
}


struct BeatDetector {
    release: f32,

    current_level: f32,
    max_level: f32
}

impl BeatDetector {
    fn new(sample_rate: f32, release_time: f32) -> Self {
        BeatDetector {
            release: (-1.0 / (sample_rate * release_time)).exp(),

            current_level: 0.0,
            max_level: 0.0
        }
    }

    fn process(&mut self, level: f32) -> f32 {
        if level >= self.current_level {
            self.current_level = level;
            self.max_level = self.current_level
        } else {
            self.current_level = self.release * (self.current_level - level) + level;
        }
        self.current_level
    }

    fn max_level(&self) -> f32 {
        self.max_level
    }

    fn level(&self) -> f32 {
        self.current_level
    }
}


#[derive(PartialEq, Clone, Copy, Debug)]
enum State {
    Attack,
    Release,
    Idle,
    Disabled
}

use State::*;

#[uri("http://johannes-mueller.org/lv2/envolvigo#lv2")]
struct Envolvigo {
    urids: urids::URIDs,
    ui_active: bool,
    ui_notified: bool,

    sample_rate: f32,

    beat_detector: BeatDetector,

    attack_smooth: EnvelopeDetector,
    sustain_smooth: EnvelopeDetector,

    attack_slow: EnvelopeDetector,
    attack_fast: EnvelopeDetector,

    release_slow: EnvelopeDetector,
    release_fast: EnvelopeDetector,

    attack_boost: Dezipper,
    sustain_boost: Dezipper,

    result_gain: EnvelopeDetector,

    outgain: Dezipper,
    mix: Dezipper,

    gain_buffer: Vec<f32>,
    input_buffer: Vec<f32>,

    state: State,
}

impl Plugin for Envolvigo {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        let sample_rate = plugin_info.sample_rate() as f32;
        let urids: urids::URIDs = features.map.populate_collection()?;
        let max_block_length = features
            .options
            .retrieve_option(urids.max_block_length)
            .and_then(|atom| atom.read(urids.atom.int, ()))
            .unwrap_or(8192) as usize;

        Some(Self {
            ui_active: false,
            ui_notified: false,
            urids,

            sample_rate,

            beat_detector: BeatDetector::new(sample_rate, 0.2),

            attack_smooth: EnvelopeDetector::new(sample_rate),
            sustain_smooth: EnvelopeDetector::new(sample_rate),

            attack_slow: EnvelopeDetector::new(sample_rate),
            attack_fast: EnvelopeDetector::new(sample_rate),

            release_slow: EnvelopeDetector::new(sample_rate),
            release_fast: EnvelopeDetector::new(sample_rate),

            attack_boost: Dezipper::new(0.0, sample_rate),
            sustain_boost: Dezipper::new(0.0, sample_rate),

            result_gain: EnvelopeDetector::new(sample_rate),

            outgain: Dezipper::new(1.0, sample_rate),
            mix: Dezipper::new(1.0, sample_rate),

            gain_buffer: Vec::with_capacity(max_block_length),
            input_buffer: Vec::with_capacity(max_block_length),

            state: Idle,
        })
    }

    fn run(&mut self, ports: &mut Ports, _features: &mut ()) {
        self.attack_fast.set_params(0.0, 0.02);
        self.attack_slow.set_params(0.02, 5.0);

        self.release_fast.set_params(0.01, 0.02);
        self.release_slow.set_params(0.02, 0.025);

        self.attack_boost.set_value(ports.attack_boost.max(-30.0).min(30.0));
        self.sustain_boost.set_value(ports.sustain_boost.max(-30.0).min(30.0));

        self.attack_smooth.set_params(0.0, ports.attack_smooth.max(0.0001).min(0.05));
        let sustain_smooth = ports.sustain_smooth.max(0.001).min(0.2);
        self.sustain_smooth.set_params(sustain_smooth, sustain_smooth);

        let (mut state, mix) = if *ports.enabled > 0.5 {
	    (
                match self.state {
                    Disabled => Idle,
                    state => state
                },
                ports.mix.max(0.0).min(1.0)
            )
	} else {
	    (
                Disabled,
                0.0
            )
        };

        let sidechain = *ports.use_sidechain > 0.5;

        self.outgain.set_value(from_dB(ports.outgain.max(-60.0).min(6.0)));
        self.mix.set_value(mix);

        self.check_notification_events(ports);

        self.gain_buffer.clear();

        let mut attack_point: Option<usize> = None;
        let mut release_point: Option<usize> = None;
        let mut idle_point: Option<usize> = None;

        if self.ui_active {
            self.input_buffer.clear();
            self.input_buffer.extend(ports.input.iter().map(to_dB));
        }

        for ((sample_num, in_frame), out_frame, sidechain_in) in izip!(
            ports.input.iter().enumerate(), ports.output.iter_mut(),
            ports.sidechain_input.iter(),
        ) {
            let attack_boost = self.attack_boost.process();
            let sustain_boost = self.sustain_boost.process();

            let lvl = if sidechain {
                sidechain_in
            } else {
                in_frame
            }.abs();


            let old_lvl = self.beat_detector.level();
            //println!("{} {} {}", lvl, old_lvl, in_frame);
            let beat_detect = self.beat_detector.process(lvl);

            if beat_detect > old_lvl && state != Disabled {
                if state != Attack {
                    self.attack_fast.reset(0.0);
                    self.attack_slow.reset(0.0);
                    self.attack_smooth.reset(self.result_gain.level());
                    //println!("ATK {} {}", lvl, old_lvl);
                    if attack_point.is_none() {
                        attack_point = Some(sample_num);
                    }
                }
                state = Attack;
            }

            let gain = match state {
                Attack => {
                    let atk_fast = self.attack_fast.process(lvl);
                    let atk_slow = self.attack_slow.process(lvl);
                    let delta_atk = atk_fast - atk_slow;

                    let pregain = self.attack_smooth.process(
                        from_dB(delta_atk / self.beat_detector.max_level())
                    );

                    let gain = pregain.powf(attack_boost);
                    if pregain < 1.0 {
                        //println!("REL {} {} {} {}", lvl, delta_atk, atk_fast, atk_slow);
                        state = Release;
                        release_point = Some(sample_num);
                        self.release_fast.reset(atk_slow);
                        self.release_slow.reset(0.0);
                        self.sustain_smooth.reset(pregain);
                    }
                    gain
                }
                Release => {
                    let rel_fast = self.release_fast.process(lvl);
                    let rel_slow = self.release_slow.process(lvl);

                    let delta_rel = rel_fast - rel_slow;
                    let pregain = self.sustain_smooth.process(
                        from_dB(
                            delta_rel / self.attack_slow.level()
                                * (15.0+3.0*ports.sustain_smooth.log10()) / 7.0
                            // voodoo to compensate smoothening
                        )
                    );

                    if to_dB(&pregain) < 0.0 {
                        //println!("IDLE {} {} {} {}", lvl, delta_rel, rel_fast, rel_slow);
                        idle_point = Some(sample_num);
                        state = Idle;
                    }
                    pregain.powf(sustain_boost)
                }
                Idle | Disabled => {
                    self.sustain_smooth.process(1.0)
                }
            };
            let gain = self.result_gain.process(gain);

            self.gain_buffer.push(gain);

            let out = *in_frame * gain * self.outgain.process();
            let mix = self.mix.process();

            *out_frame = out * mix + *in_frame * (1.0 - mix);
        }

        self.state = state;

        if self.ui_active {
            let mut sequence_writer = ports.notify.init(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.unit.frame)
            ).unwrap();

            if !self.ui_notified {
                let mut object_writer = sequence_writer.init(
                    TimeStamp::Frames(0),
                    self.urids.atom.object,
                    ObjectHeader {
                        id: None,
                        otype: self.urids.plugin_config.into_general(),
                    }
                ).unwrap();

                object_writer.init(self.urids.sample_rate, None,
                                   self.urids.atom.float,
                                   self.sample_rate as f32);
            }
            self.ui_notified = true;

            let mut object_writer = sequence_writer.init(
                TimeStamp::Frames(0),
                self.urids.atom.object,
                ObjectHeader {
                    id: None,
                    otype: self.urids.plugin_state.into_general(),
                }
            ).unwrap();

            if let Some(point) = attack_point {
                object_writer.init(self.urids.attack_point, None, self.urids.atom.int, point as i32);
            }
            if let Some(point) = release_point {
                object_writer.init(self.urids.release_point, None, self.urids.atom.int, point as i32);
            }
            if let Some(point) = idle_point {
                object_writer.init(self.urids.idle_point, None, self.urids.atom.int, point as i32);
            }

            let mut gain_writer: lv2_atom::vector::VectorWriter<Float> =
                object_writer.init(self.urids.gain_signal, None,
                                   self.urids.atom.vector(),
                                   self.urids.atom.float).unwrap();
            gain_writer.append(self.gain_buffer.iter().map(to_dB).collect::<Vec<f32>>().as_slice());

            let mut input_writer: lv2_atom::vector::VectorWriter<Float> =
                object_writer.init(self.urids.input_signal, None,
                                   self.urids.atom.vector(),
                                   self.urids.atom.float).unwrap();
            input_writer.append(&self.input_buffer);

            let mut output_writer: lv2_atom::vector::VectorWriter<Float> =
                object_writer.init(self.urids.output_signal, None,
                                   self.urids.atom.vector(),
                                   self.urids.atom.float).unwrap();
            output_writer.append(ports.output.iter().map(to_dB).collect::<Vec<f32>>().as_slice());
        }
    }
}

impl Envolvigo {
    fn check_notification_events(&mut self, ports: &mut Ports) {
        let control_sequence = match ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat) {
                None => return,
                Some(cs) => cs
            };

        for (_, message) in control_sequence {
            if let Some((header,  _)) = message.read(self.urids.atom.object, ()) {
                if header.otype == self.urids.ui_on {
                    self.ui_active = true;
                    self.ui_notified = false;
                } else if header.otype == self.urids.ui_off {
                    self.ui_active = false;
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn from_dB(v: f32) -> f32 {
    10.0f32.powf(0.05 * v)
}

#[allow(non_snake_case)]
fn to_dB(v: &f32) -> f32 {
    20.0f32 * f32::log10(v.abs().max(1e-8))
}

fn no_denormal(v: f32) -> f32 {
    if v.is_normal() {
        v
    } else {
        0.0
    }
}

lv2_descriptors!(Envolvigo);
