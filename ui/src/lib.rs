use std::sync::{Arc, RwLock};

#[macro_use]
extern crate cascade;

#[macro_use] extern crate pugl_ui;

use lv2::prelude::*;
use lv2_ui::*;
use lv2;

use jilar;

use urids;

use pugl_ui as pugl;
use pugl_ui::layout;
use pugl_ui::widget;
use pugl_ui::widget::Widget;
use pugl_sys as pugl_sys;
use pugl_sys::pugl::PuglViewTrait;

#[derive(FeatureCollection)]
struct Features<'a> {
    map: LV2Map<'a>
}


struct UIPorts {
    enabled: ControlPort,
    use_sidechain: ControlPort,
    attack_boost: ControlPort,
    attack_smooth: ControlPort,
    sustain_boost: ControlPort,
    sustain_smooth: ControlPort,
    gain_attack: ControlPort,
    gain_release: ControlPort,
    outgain: ControlPort,
    mix: ControlPort,
    control: UIAtomPort,
    notify: UIAtomPort
}

impl UIPorts {
    fn new(urid: URID<AtomEventTransfer>) -> Self {
        UIPorts {
            enabled: ControlPort::new(0),
            use_sidechain: ControlPort::new(1),
            attack_boost: ControlPort::new(2),
            attack_smooth: ControlPort::new(3),
            sustain_boost: ControlPort::new(4),
            sustain_smooth: ControlPort::new(5),
            gain_attack: ControlPort::new(6),
            gain_release: ControlPort::new(7),
            outgain: ControlPort::new(8),
            mix: ControlPort::new(9),
            control: UIAtomPort::new(urid, 10),
            notify: UIAtomPort::new(urid, 11)
        }
    }
}

impl UIPortsTrait for UIPorts {
    fn map_control_port(&mut self, port_index: u32) -> Option<&mut ControlPort> {
        match port_index {
            0 => Some(&mut self.enabled),
            1 => Some(&mut self.use_sidechain),
            2 => Some(&mut self.attack_boost),
            3 => Some(&mut self.attack_smooth),
            4 => Some(&mut self.sustain_boost),
            5 => Some(&mut self.sustain_smooth),
            6 => Some(&mut self.gain_attack),
            7 => Some(&mut self.gain_release),
            8 => Some(&mut self.outgain),
            9 => Some(&mut self.mix),
            _ => None
        }
    }

    fn map_atom_port(&mut self, port_index: u32) -> Option<&mut UIAtomPort> {
        match port_index {
            10 => Some(&mut self.control),
            11 => Some(&mut self.notify),
            _ => None
        }
    }
}


#[uri("https://johannes-mueller.org/lv2/envolvigo#ui")]
struct EnvolvigoUI {
    view: Box<pugl_sys::PuglView<pugl::ui::UI<RootWidget>>>,

    enabled_button: widget::WidgetHandle<jilar::Button>,
    use_sidechain_button: widget::WidgetHandle<jilar::Button>,

    attack_boost_dial: widget::WidgetHandle<jilar::Dial>,
    attack_smooth_dial: widget::WidgetHandle<jilar::Dial>,

    sustain_boost_dial: widget::WidgetHandle<jilar::Dial>,
    sustain_smooth_dial: widget::WidgetHandle<jilar::Dial>,

    outgain_dial: widget::WidgetHandle<jilar::Dial>,
    mix_dial: widget::WidgetHandle<jilar::Dial>,

    osci: widget::WidgetHandle<jilar::Osci>,

    ports: UIPorts,
    write_handle: PluginPortWriteHandle,

    input_signal: Arc<RwLock<Vec<f32>>>,
    output_signal: Arc<RwLock<Vec<f32>>>,
    gain_signal: Arc<RwLock<Vec<f32>>>,
    attack_point: Arc<RwLock<Option<usize>>>,
    release_point: Arc<RwLock<Option<usize>>>,
    idle_point: Arc<RwLock<Option<usize>>>,

    sample_rate: f64,
    display_time: Arc<RwLock<f64>>,
    drawing_task_submitted: bool,

    urids: urids::URIDs
}

impl EnvolvigoUI {
    fn new(features: &mut Features<'static>,
           parent_window: *mut std::ffi::c_void,
           write_handle: PluginPortWriteHandle) -> Option<Self> {
        let mut ui = Box::new(pugl::ui::UI::new(Box::new(RootWidget::default())));

        let enabled_button = ui.new_widget(jilar::Button::new_toggle_button("Enabled"));
        let use_sidechain_button = ui.new_widget(jilar::Button::new_toggle_button("Sidechain"));

        let attack_boost_dial = ui.new_widget( cascade! {
            jilar::Dial::new(-30.0, 30.0, 5.0);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(0.0);
            ..set_hue(Some(0.1));
            ..set_formater(&|v| format!("{:.1} dB", v));
        });
        let attack_smooth_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.0001, 0.05, 0.01);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(0.035);
            ..set_hue(Some(0.1));
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });

        let sustain_boost_dial = ui.new_widget( cascade! {
            jilar::Dial::new(-30.0, 30.0, 5.0);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(0.0);
            ..set_hue(Some(0.7));
            ..set_formater(&|v| format!("{:.1} dB", v));
        });
        let sustain_smooth_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.001, 0.2, 0.01);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(0.035);
            ..set_hue(Some(0.7));
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });

        let outgain_dial = ui.new_widget( cascade! {
            jilar::Dial::new(-60.0, 6.0, 6.0);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(0.0);
            ..set_formater(&|v| format!("{:.1} dB", v));
        });

        let mix_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.0, 1.0, 0.1);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_default_value(1.0);
            ..set_formater(&|v| format!("{:.0} %", v*100.0));
        });

        let osci = ui.new_widget( cascade! {
            jilar::Osci::new();
            ..set_level_range(-72.0, 12.0);
            ..set_min_height(180.0);
            ..linear_major_xticks(10);
            ..linear_major_yticks(12);
        });

        ui.layouter_handle(ui.root_layout()).set_padding(5.0);
        ui.pack_to_layout(osci, ui.root_layout(), layout::StackDirection::Back);

        let controls_layout = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.widget(controls_layout.widget()).lock_height();
        ui.pack_to_layout(controls_layout.widget(), ui.root_layout(), layout::StackDirection::Back);

        // Layout "Enabled" and "Sidechain"
        let vl = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(vl.widget(), controls_layout, layout::StackDirection::Back);

        ui.add_spacer(vl, layout::StackDirection::Back);
        ui.pack_to_layout(enabled_button, vl, layout::StackDirection::Back);
        ui.pack_to_layout(use_sidechain_button, vl, layout::StackDirection::Back);
        ui.add_spacer(vl, layout::StackDirection::Back);

        ui.add_spacer(controls_layout, layout::StackDirection::Back);

        // Layout "Attack dials"
        let sect_layout = ui.new_layouter::<layout::VerticalLayouter>();
        ui.widget(sect_layout.widget()).lock_width();
        ui.pack_to_layout(sect_layout.widget(), controls_layout, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(attack_boost_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Attack boost"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(sect_layout, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(attack_smooth_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Smooth"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(controls_layout, layout::StackDirection::Back);

        // Layout "Sustain dials"
        let sect_layout = ui.new_layouter::<layout::VerticalLayouter>();
        ui.widget(sect_layout.widget()).lock_width();
        ui.pack_to_layout(sect_layout.widget(), controls_layout, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(sustain_boost_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Sustain boost"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(sect_layout, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(sustain_smooth_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Time"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        // Layout "Outgain Mix dials"
        let sect_layout = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(sect_layout.widget(), controls_layout, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(outgain_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Output level"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(mix_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), sect_layout, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Dry/Wet"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.do_layout();

        let view = pugl_sys::PuglView::make_view(ui, parent_window);

        let ui = view.handle();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.make_resizable();
        ui.set_window_title("Envolvigo – a Transient Designer");
        ui.show_window();

        let urids: urids::URIDs = features.map.populate_collection()?;
        let ports = UIPorts::new(urids.atom_event_transfer);
        Some(Self {
            view,
            enabled_button,
            use_sidechain_button,
            attack_boost_dial,
            sustain_boost_dial,
            attack_smooth_dial,
            sustain_smooth_dial,
            outgain_dial,
            mix_dial,
            osci,
            ports,
            write_handle,
            input_signal: Arc::new(RwLock::new(Vec::new())),
            output_signal: Arc::new(RwLock::new(Vec::new())),
            gain_signal: Arc::new(RwLock::new(Vec::new())),
            attack_point: Arc::new(RwLock::new(None)),
            release_point: Arc::new(RwLock::new(None)),
            idle_point: Arc::new(RwLock::new(None)),
            sample_rate: 0.0,
            display_time: Arc::new(RwLock::new(0.25)),
            drawing_task_submitted: false,
            urids
        })
    }

    fn ui(&self) -> &mut pugl::ui::UI<RootWidget> {
        self.view.handle()
    }

    fn send_ui_enable(&mut self) {
        println!("ui_enable");
        self.ports.control.init(
            self.urids.atom.object,
            ObjectHeader {
                id: None,
                otype: self.urids.ui_on.into_general()
            });
        self.write_handle.write_port(&self.ports.control);
    }

    fn send_ui_disable(&mut self) {
        println!("ui_disable");
        self.ports.control.init(
            self.urids.atom.object,
            ObjectHeader {
                id: None,
                otype: self.urids.ui_off.into_general()
            });
        self.write_handle.write_port(&self.ports.control);
    }
}

impl lv2_ui::PluginUI for EnvolvigoUI {

    type InitFeatures = Features<'static>;
    type UIPorts = UIPorts;

    fn new(_plugin_ui_info: &PluginUIInfo,
           features: &mut Self::InitFeatures,
           parent_window: *mut std::ffi::c_void,
           write_handle: PluginPortWriteHandle) -> Option<Self> {
        let mut ui = Self::new(features, parent_window, write_handle)?;
        ui.send_ui_enable();
        Some(ui)
    }

    fn cleanup(&mut self) {
        self.send_ui_disable();
    }

    fn ports(&mut self) -> &mut UIPorts {
        &mut self.ports
    }

    fn widget(&self) -> lv2_sys::LV2UI_Widget {
        self.view.native_window() as lv2_sys::LV2UI_Widget
    }

    fn idle(&mut self) -> i32 {
        let ui = self.ui();
        ui.next_event(0.0);

        if ui.close_request_issued() {
            return 1;
        }

        if ui.root_widget().focus_next() {
                ui.focus_next_widget();
        }

        if let Some(ts) = self.ui().widget(self.enabled_button).changed_toggle_state() {
            self.ports.enabled.set_value(if ts { 1.0 } else { 0.0 });
            self.write_handle.write_port(&self.ports.enabled);
        }
        if let Some(ts) = self.ui().widget(self.use_sidechain_button).changed_toggle_state() {
            self.ports.use_sidechain.set_value(if ts { 1.0 } else { 0.0 });
            self.write_handle.write_port(&self.ports.use_sidechain);
        }

        if let Some(v) = self.ui().widget(self.attack_boost_dial).changed_value() {
            self.ports.attack_boost.set_value(v as f32);
            self.write_handle.write_port(&self.ports.attack_boost);
        }
        if let Some(v) = self.ui().widget(self.attack_smooth_dial).changed_value() {
            self.ports.attack_smooth.set_value(v as f32);
            self.write_handle.write_port(&self.ports.attack_smooth);
        }

        if let Some(v) = self.ui().widget(self.sustain_boost_dial).changed_value() {
            self.ports.sustain_boost.set_value(v as f32);
            self.write_handle.write_port(&self.ports.sustain_boost);
        }
        if let Some(v) = self.ui().widget(self.sustain_smooth_dial).changed_value() {
            self.ports.sustain_smooth.set_value(v as f32);
            self.write_handle.write_port(&self.ports.sustain_smooth);
        }

        if let Some(v) = self.ui().widget(self.outgain_dial).changed_value() {
            self.ports.outgain.set_value(v as f32);
            self.write_handle.write_port(&self.ports.outgain);
        }
        if let Some(v) = self.ui().widget(self.mix_dial).changed_value() {
            self.ports.mix.set_value(v as f32);
            self.write_handle.write_port(&self.ports.mix);
        }

        self.update();

        0
    }

    fn update(&mut self) {
        if let Some(v) = self.ports.enabled.changed_value() {
            self.ui().widget(self.enabled_button).set_toggle_state(v > 0.5);
        }
        if let Some(v) = self.ports.use_sidechain.changed_value() {
            self.ui().widget(self.use_sidechain_button).set_toggle_state(v > 0.5);
        }

        if let Some(v) = self.ports.attack_boost.changed_value() {
            self.ui().widget(self.attack_boost_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.attack_smooth.changed_value() {
            self.ui().widget(self.attack_smooth_dial).set_value(v as f64);
        }

        if let Some(v) = self.ports.sustain_boost.changed_value() {
            self.ui().widget(self.sustain_boost_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.sustain_smooth.changed_value() {
            self.ui().widget(self.sustain_smooth_dial).set_value(v as f64);
        }

        if let Some(v) = self.ports.outgain.changed_value() {
            self.ui().widget(self.outgain_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.mix.changed_value() {
            self.ui().widget(self.mix_dial).set_value(v as f64);
        }

        let mut osci_repaint = false;
        let mut received_sample_rate = false;
        let displayed_sample_num = (*self.display_time.read().unwrap() * self.sample_rate).ceil() as usize;

        if let Some((_, object_reader)) = self.ports.notify.read(self.urids.atom.object, ()) {
            for (header, atom) in object_reader {
                if header.key == self.urids.sample_rate  {
                    if let Some(sr) =  atom.read(self.urids.atom.float, ()) {
                        self.sample_rate = sr as f64;
                        received_sample_rate = true;
                    } else {
                        eprintln!("expected float for sample rate, got something different");
                    };
                } else if header.key == self.urids.attack_point {
                    if let Some(ap) = atom.read(self.urids.atom.int, ()) {
                        let mut attack_point = self.attack_point.write().unwrap();
                        *attack_point = Some(ap as usize);
                        let mut input_signal = self.input_signal.write().unwrap();
                        let mut output_signal = self.output_signal.write().unwrap();
                        let mut gain_signal = self.gain_signal.write().unwrap();

                        let cut_samples = input_signal.len() - (0.01 * self.sample_rate).floor() as usize;
                        if input_signal.len() != gain_signal.len() {
                            println!("warning: input != gain {} {}", input_signal.len(), gain_signal.len());
                        }
                        println!("received attack point {} {} {} {}", ap, cut_samples, gain_signal.len(),
                                 input_signal.iter().fold(-160.0f32, |a, v| a.max(*v)));

                        gain_signal.drain(..cut_samples);
                        input_signal.drain(..cut_samples);
                        output_signal.drain(..cut_samples);
                    } else {
                        eprintln!("expected int for attack point, got something different");
                    };
                } else if header.key == self.urids.release_point {
                    if let Some(p) = atom.read(self.urids.atom.int, ()) {
                        let mut release_point = self.release_point.write().unwrap();
                        let mut input_signal = self.input_signal.read().unwrap();
                        *release_point = Some(p as usize + input_signal.len());
                    } else {
                        eprintln!("expected int for release point, got something different");
                    };
                } else if header.key == self.urids.idle_point {
                    if let Some(p) = atom.read(self.urids.atom.int, ()) {
                        let mut idle_point = self.idle_point.write().unwrap();
                        let mut input_signal = self.input_signal.read().unwrap();
                        *idle_point = Some(p as usize + input_signal.len());
                    } else {
                        eprintln!("expected int for idle point, got something different");
                    };
                } else if header.key == self.urids.gain_signal {
                    if let Some(new_gain_signal) = atom.read(self.urids.atom.vector(), self.urids.atom.float) {
                        let mut gain_signal = self.gain_signal.write().unwrap();

                        if gain_signal.len() < displayed_sample_num {
                            gain_signal.extend(new_gain_signal);
                        }
                        //println!("{} gain samples", gain_signal.len());
                        osci_repaint = true;
                    } else {
                        eprintln!("expected vector of floats, got something different");
                    }
                } else if header.key == self.urids.input_signal {
                    if let Some(new_input_signal) = atom.read(self.urids.atom.vector(), self.urids.atom.float) {
                        let mut input_signal = self.input_signal.write().unwrap();

                        if input_signal.len() < displayed_sample_num {
                            input_signal.extend(new_input_signal);
                        }
                        osci_repaint = true;
                    } else {
                        eprintln!("expected vector of floats, got something different");
                    }
                } else if header.key == self.urids.output_signal {
                    if let Some(new_output_signal) = atom.read(self.urids.atom.vector(), self.urids.atom.float) {
                        let mut output_signal = self.output_signal.write().unwrap();

                        if output_signal.len() < displayed_sample_num {
                            output_signal.extend(new_output_signal);
                        }
                        osci_repaint = true;
                    } else {
                        eprintln!("expected vector of floats, got something different");
                    }
                }
            }
        }

        if received_sample_rate && !self.drawing_task_submitted {
            self.ui().widget(self.osci).submit_draw_task(
                Box::new(OsciDrawings {
                    input_signal: self.input_signal.clone(),
                    output_signal: self.output_signal.clone(),
                    gain_signal: self.gain_signal.clone(),
                    sample_rate: self.sample_rate,
                    display_time: self.display_time.clone(),
                    attack_point: self.attack_point.clone(),
                    release_point: self.release_point.clone(),
                    idle_point: self.idle_point.clone()
                })
            );
            self.drawing_task_submitted = true;
            osci_repaint = true;
        }

        if osci_repaint {
            self.ui().widget(self.osci).ask_for_repaint();
        }
    }
}


unsafe impl PluginUIInstanceDescriptor for EnvolvigoUI {
    const DESCRIPTOR: lv2_sys::LV2UI_Descriptor = lv2_sys::LV2UI_Descriptor {
        URI: Self::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
        instantiate: Some(PluginUIInstance::<Self>::instantiate),
        cleanup: Some(PluginUIInstance::<Self>::cleanup),
        port_event: Some(PluginUIInstance::<Self>::port_event),
        extension_data: Some(PluginUIInstance::<Self>::extension_data)

    };
}

#[no_mangle]
pub unsafe extern "C" fn lv2ui_descriptor(index: u32) -> *const lv2_sys::LV2UI_Descriptor {
    match index {
        0 => &<EnvolvigoUI as PluginUIInstanceDescriptor>::DESCRIPTOR,
        _ => std::ptr::null()
    }
}


#[derive(Default)]
struct RootWidget {
    stub: pugl::widget::WidgetStub,
    focus_next: bool
}

impl pugl::widget::Widget for RootWidget {
    widget_stub!();
    fn exposed (&self, _expose: &pugl_sys::ExposeArea, cr: &cairo::Context) {
        cr.set_source_rgb (0.2, 0.2, 0.2);
        let size = self.size();
        cr.rectangle (0., 0., size.w, size.h);
        cr.fill ();
    }
    fn event(&mut self, ev: pugl_sys::Event) -> Option<pugl_sys::Event> {
        ev.try_keypress()
            .and_then(|kp| kp.try_char())
            .and_then(|c| {
                match c {
                    '\t' => {
                        self.focus_next = true;
                        event_processed!()
                    },
                    _ => event_not_processed!()
                }
            })
            .or(event_not_processed!()).and_then (|p| p.pass_event (ev))
    }
}

impl RootWidget {
    pub fn focus_next(&mut self) -> bool {
        let f = self.focus_next;
        self.focus_next = false;
        f
    }
}

struct OsciDrawings {
    input_signal: Arc<RwLock<Vec<f32>>>,
    output_signal: Arc<RwLock<Vec<f32>>>,
    gain_signal: Arc<RwLock<Vec<f32>>>,
    sample_rate: f64,
    display_time: Arc<RwLock<f64>>,
    attack_point: Arc<RwLock<Option<usize>>>,
    release_point: Arc<RwLock<Option<usize>>>,
    idle_point: Arc<RwLock<Option<usize>>>
}

impl jilar::osci::DrawingTask for OsciDrawings {
    fn draw(&self, osci: &jilar::osci::Osci, cr: &cairo::Context) {
        let input_signal = self.input_signal.read().unwrap();
        let output_signal = self.output_signal.read().unwrap();
        let gain_signal = self.gain_signal.read().unwrap();
        let display_time = *self.display_time.read().unwrap();

        let samples_per_pixel =
            (self.sample_rate * display_time / osci.size().w)
            .ceil() as usize;

        let attack_point = match *self.attack_point.read().unwrap() {
            Some(ap) => ap,
            None => return
        };

        let left = osci.pos().x;
        let top = osci.pos().y;
        let width = osci.size().w;
        let height = osci.size().h;
        let right = left + width;
        let bottom = top + height;

        /*
        cr.set_source_rgba(0.0, 0.0, 1.0, 0.4);
        cr.move_to(left, bottom);

        let mut x = left;
        for chunk in gain_signal.chunks(samples_per_pixel) {
            let val = (chunk.iter().sum::<f32>()/chunk.len() as f32) as f64;
            cr.line_to(x, osci.scale_y(val));
            x += 1.0;
            if x > right {
                break
            }
        }

        cr.line_to(right, osci.scale_y(0.0));
        cr.line_to(right, bottom);
        cr.fill();
        */
        cr.set_source_rgba(0.4, 0.4, 0.4, 0.4);
        cr.set_line_width(0.5);
        cr.set_line_join(cairo::LineJoin::Round);

        cr.move_to(left, bottom);
        let mut x = left;
        for chunk in input_signal[attack_point..].chunks(samples_per_pixel) {
            let max = (chunk.iter().sum::<f32>()/chunk.len() as f32) as f64;
            let max = (chunk.iter().fold(-160.0f32, |acc, &v| acc.max(v))) as f64;

            cr.line_to(x, osci.scale_y(max));

            x += 1.0;
            if x > right {
                break
            }
        }
        cr.line_to(x-1.0, bottom);
        cr.fill();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(0.25);
        cr.set_line_join(cairo::LineJoin::Round);

        let mut x = left;
        for chunk in output_signal[attack_point..].chunks(samples_per_pixel) {
            let val = (chunk.iter().sum::<f32>()/chunk.len() as f32) as f64;
            let max = (chunk.iter().fold(-160.0f32, |acc, &v| acc.max(v))) as f64;
            cr.line_to(x, osci.scale_y(max));

            x += 1.0;
            if x > right {
                break
            }
        }
        cr.stroke();


        if let Some(release_point) = *self.release_point.read().unwrap() {
            cr.set_source_rgb(1.0, 0.0, 0.0);
            cr.set_line_width(0.25);
            cr.move_to((release_point-attack_point) as f64/samples_per_pixel as f64, top);
            cr.line_to((release_point-attack_point) as f64/samples_per_pixel as f64, bottom);
            cr.stroke();
        }

        if let Some(idle_point) = *self.idle_point.read().unwrap() {
            cr.set_source_rgb(0.0, 1.0, 0.0);
            cr.set_line_width(0.25);
            cr.move_to((idle_point-attack_point) as f64/samples_per_pixel as f64, top);
            cr.line_to((idle_point-attack_point) as f64/samples_per_pixel as f64, bottom);
            cr.stroke();
        }
    }
}
