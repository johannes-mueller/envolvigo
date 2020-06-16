use lv2::prelude::*;
use lv2_ui::*;


#[uri("http://lv2plug.in/ns/ext/parameters#sample_rate")]
pub struct SampleRate;

#[uri("http://johannes-mueller.org/lv2/envolvigo#PluginConfig")]
pub struct PluginConfig;

#[uri("http://johannes-mueller.org/lv2/envolvigo#ui_on")]
pub struct UIOn;

#[uri("http://johannes-mueller.org/lv2/envolvigo#ui_off")]
pub struct UIOff;

#[uri("http://lv2plug.in/ns/ext/buf-size#minBlockLength")]
pub struct MaxBlockLength;

#[uri("http://johannes-mueller.org/lv2/envolvigo#AudioData")]
pub struct AudioData;

#[uri("http://johannes-mueller.org/lv2/envolvigo#PluginState")]
pub struct PluginState;

#[uri("http://johannes-mueller.org/lv2/envolvigo#gain_signal")]
pub struct GainSignal;

#[derive(URIDCollection)]
pub struct URIDs {
    pub atom: AtomURIDCollection,
    pub unit: UnitURIDCollection,
    pub plugin_config: URID<PluginConfig>,
    pub sample_rate: URID<SampleRate>,
    pub ui_on: URID<UIOn>,
    pub ui_off: URID<UIOff>,
    pub atom_event_transfer: URID<AtomEventTransfer>,
    pub max_block_length: URID<MaxBlockLength>,
    pub plugin_state: URID<PluginState>,
    pub audio_data: URID<AudioData>,
    pub gain_signal: URID<GainSignal>
}
