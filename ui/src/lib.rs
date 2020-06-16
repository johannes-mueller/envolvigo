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
    sustain_boost: ControlPort,
    attack_time: ControlPort,
    attack_release: ControlPort,
    sustain_time: ControlPort,
    sustain_attack: ControlPort,
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
            attack_time: ControlPort::new(3),
            attack_release: ControlPort::new(4),
            sustain_boost: ControlPort::new(5),
            sustain_time: ControlPort::new(6),
            sustain_attack: ControlPort::new(7),
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
            3 => Some(&mut self.attack_time),
            4 => Some(&mut self.attack_release),
            5 => Some(&mut self.sustain_boost),
            6 => Some(&mut self.sustain_time),
            7 => Some(&mut self.sustain_attack),
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
    attack_time_dial: widget::WidgetHandle<jilar::Dial>,
    attack_release_dial: widget::WidgetHandle<jilar::Dial>,

    sustain_boost_dial: widget::WidgetHandle<jilar::Dial>,
    sustain_time_dial: widget::WidgetHandle<jilar::Dial>,
    sustain_attack_dial: widget::WidgetHandle<jilar::Dial>,

    outgain_dial: widget::WidgetHandle<jilar::Dial>,
    mix_dial: widget::WidgetHandle<jilar::Dial>,

    osci: widget::WidgetHandle<jilar::Osci>,

    ports: UIPorts,
    write_handle: PluginPortWriteHandle,

    gain_signal: Arc<RwLock<Vec<f32>>>,

    sample_rate: f32,
    display_time: f32,

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
            ..set_hue(Some(0.1));
            ..set_large();
            ..set_formater(&|v| format!("{:.1} dB", v));
        });
        let attack_time_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.01, 0.4, 0.05);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_hue(Some(0.1));
            ..set_small();
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });
        let attack_release_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.001, 0.4, 0.05);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_hue(Some(0.1));
            ..set_small();
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });

        let sustain_boost_dial = ui.new_widget( cascade! {
            jilar::Dial::new(-30.0, 30.0, 5.0);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_hue(Some(0.7));
            ..set_large();
            ..set_formater(&|v| format!("{:.1} dB", v));
        });
        let sustain_time_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.01, 5.0, 0.5);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_hue(Some(0.7));
            ..set_small();
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });
        let sustain_attack_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.01, 0.4, 0.05);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_hue(Some(0.7));
            ..set_small();
            ..set_formater(&|v| format!("{:.1} ms", v*1000.));
        });

        let outgain_dial = ui.new_widget( cascade! {
            jilar::Dial::new(-60.0, 6.0, 6.0);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_formater(&|v| format!("{:.1} dB", v));
        });

        let mix_dial = ui.new_widget( cascade! {
            jilar::Dial::new(0.0, 1.0, 0.1);
            ..set_plate_draw( &|d: &jilar::Dial, cr: &cairo::Context| {
                jilar::dial::draw_angle_tics(d, cr, 11)
            });
            ..set_formater(&|v| format!("{:.0} %", v*100.0));
        });

        let gain_signal = Arc::new(RwLock::new(Vec::new()));

        let osci = ui.new_widget( cascade! {
            jilar::Osci::new();
            ..set_level_range(-24.0, 24.0);
            ..set_min_height(180.0);
            ..linear_major_xticks(10);
            ..linear_major_yticks(12);
            ..submit_draw_task(Box::new(OsciDrawings {
                gain_signal: gain_signal.clone(),
                sample_rate: 48000.0,
                display_time: 1.0
            }));
        });

        ui.pack_to_layout(osci, ui.root_layout(), layout::StackDirection::Back);

        let controls_layout = ui.new_layouter::<layout::HorizontalLayouter>();
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

        let subsect_layout = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(subsect_layout.widget(), sect_layout, layout::StackDirection::Back);

        let vl = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(vl.widget(), subsect_layout, layout::StackDirection::Back);
        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(attack_time_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Time"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(subsect_layout, layout::StackDirection::Back);

        let vl = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(vl.widget(), subsect_layout, layout::StackDirection::Back);
        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(attack_release_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Release"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(controls_layout, layout::StackDirection::Back);

        // Layout "Sustain dials"
        let sect_layout = ui.new_layouter::<layout::VerticalLayouter>();
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

        let subsect_layout = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(subsect_layout.widget(), sect_layout, layout::StackDirection::Back);

        let vl = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(vl.widget(), subsect_layout, layout::StackDirection::Back);
        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(sustain_time_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Time"));
        ui.pack_to_layout(lb, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        ui.add_spacer(subsect_layout, layout::StackDirection::Back);

        let vl = ui.new_layouter::<layout::VerticalLayouter>();
        ui.pack_to_layout(vl.widget(), subsect_layout, layout::StackDirection::Back);
        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        ui.pack_to_layout(sustain_attack_dial, hl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);

        let hl = ui.new_layouter::<layout::HorizontalLayouter>();
        ui.pack_to_layout(hl.widget(), vl, layout::StackDirection::Back);
        ui.add_spacer(hl, layout::StackDirection::Back);
        let lb = ui.new_widget(jilar::Label::new("Attack"));
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

        ui.add_spacer(subsect_layout, layout::StackDirection::Back);

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
        ui.set_window_title("Envolvigo – a Transient Designer");
        ui.show_window();

        let urids: urids::URIDs = features.map.populate_collection()?;
        let ports = UIPorts::new(urids.atom_event_transfer);
        Some(Self {
            view,
            enabled_button,
            use_sidechain_button,
            attack_boost_dial,
            attack_time_dial,
            attack_release_dial,
            sustain_boost_dial,
            sustain_time_dial,
            sustain_attack_dial,
            outgain_dial,
            mix_dial,
            osci,
            ports,
            write_handle,
            gain_signal,
            sample_rate: 48000.0,
            display_time: 1.0,
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
        if let Some(v) = self.ui().widget(self.attack_time_dial).changed_value() {
            self.ports.attack_time.set_value(v as f32);
            self.write_handle.write_port(&self.ports.attack_time);
        }
        if let Some(v) = self.ui().widget(self.attack_release_dial).changed_value() {
            self.ports.attack_release.set_value(v as f32);
            self.write_handle.write_port(&self.ports.attack_release);
        }

        if let Some(v) = self.ui().widget(self.sustain_boost_dial).changed_value() {
            self.ports.sustain_boost.set_value(v as f32);
            self.write_handle.write_port(&self.ports.sustain_boost);
        }
        if let Some(v) = self.ui().widget(self.sustain_time_dial).changed_value() {
            self.ports.sustain_time.set_value(v as f32);
            self.write_handle.write_port(&self.ports.sustain_time);
        }
        if let Some(v) = self.ui().widget(self.sustain_attack_dial).changed_value() {
            self.ports.sustain_attack.set_value(v as f32);
            self.write_handle.write_port(&self.ports.sustain_attack);
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
        if let Some(v) = self.ports.attack_time.changed_value() {
            self.ui().widget(self.attack_time_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.attack_release.changed_value() {
            self.ui().widget(self.attack_release_dial).set_value(v as f64);
        }


        if let Some(v) = self.ports.sustain_boost.changed_value() {
            self.ui().widget(self.sustain_boost_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.sustain_time.changed_value() {
            self.ui().widget(self.sustain_time_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.sustain_attack.changed_value() {
            self.ui().widget(self.sustain_attack_dial).set_value(v as f64);
        }

        if let Some(v) = self.ports.outgain.changed_value() {
            self.ui().widget(self.outgain_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.mix.changed_value() {
            self.ui().widget(self.mix_dial).set_value(v as f64);
        }

        let mut osci_repaint = false;

        if let Some((_, object_reader)) = self.ports.notify.read(self.urids.atom.object, ()) {
            for (header, atom) in object_reader {
                if header.key == self.urids.sample_rate  {
                    if let Some(sr) =  atom.read(self.urids.atom.float, ()) {
                        self.sample_rate = sr;
                    } else {
                        eprintln!("expected float for sample rate, got something different");
                    };
                } else if header.key == self.urids.gain_signal {
                    if let Some(new_gain_signal) = atom.read(self.urids.atom.vector(), self.urids.atom.float) {
                        let nsamples = new_gain_signal.len();
                        let mut gain_signal = self.gain_signal.write().unwrap();
                        gain_signal.extend(new_gain_signal.iter().map(to_dB));
                        if gain_signal.len() - nsamples
                            > (self.display_time * self.sample_rate).ceil() as usize {
                                gain_signal.drain(..nsamples);
                            }
                        osci_repaint = true;
                    } else {
                        eprintln!("expected vector of floats, got something different");
                    }
                }
            }
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
    gain_signal: Arc<RwLock<Vec<f32>>>,
    sample_rate: f64,
    display_time: f64,
}

impl jilar::osci::DrawingTask for OsciDrawings {
    fn draw(&self, osci: &jilar::osci::Osci, cr: &cairo::Context) {
        let gain_signal = self.gain_signal.read().unwrap();

        let samples_per_pixel = self.sample_rate * self.display_time / osci.size().w;

        cr.set_source_rgba(0.0, 0.0, 1.0, 0.4);
        cr.move_to(osci.pos().x, osci.pos().y + osci.size().h);

        let mut x = osci.pos().x;
        for chunk in gain_signal.chunks(samples_per_pixel.ceil() as usize) {
            let val = (chunk.iter().sum::<f32>()/chunk.len() as f32) as f64;
            cr.line_to(x, osci.scale_y(val));
            x += 1.0;
        }

        cr.line_to(osci.pos().x + osci.size().w, osci.pos().y + osci.size().h);
        cr.fill();
    }
}

#[allow(non_snake_case)]
fn to_dB(v: &f32) -> f32 {
    20.0f32 * f32::log10(*v)
}
