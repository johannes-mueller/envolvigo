use lv2::prelude::*;
use lv2_ui::*;


#[uri("http://lv2plug.in/ns/ext/parameters#sample_rate")]
pub struct SampleRate;

#[uri("http://lv2plug.in/ns/ext/buf-size#minBlockLength")]
pub struct MaxBlockLength;

#[uri("http://johannes-mueller.org/lv2/envolvigo#PluginConfig")]
pub struct PluginConfig;

#[uri("http://johannes-mueller.org/lv2/envolvigo#ui_on")]
pub struct UIOn;

#[uri("http://johannes-mueller.org/lv2/envolvigo#ui_off")]
pub struct UIOff;

#[uri("http://johannes-mueller.org/lv2/envolvigo#PluginState")]
pub struct PluginState;

#[uri("http://johannes-mueller.org/lv2/envolvigo#AudioData")]
pub struct AudioData;

#[uri("http://johannes-mueller.org/lv2/envolvigo#AttackPoint")]
pub struct AttackPoint;

#[uri("http://johannes-mueller.org/lv2/envolvigo#ReleasePoint")]
pub struct ReleasePoint;

#[uri("http://johannes-mueller.org/lv2/envolvigo#IdlePoint")]
pub struct IdlePoint;

#[uri("http://johannes-mueller.org/lv2/envolvigo#gain_signal")]
pub struct GainSignal;

#[uri("http://johannes-mueller.org/lv2/envolvigo#input_signal")]
pub struct InputSignal;

#[uri("http://johannes-mueller.org/lv2/envolvigo#output_signal")]
pub struct OutputSignal;


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
    pub attack_point: URID<AttackPoint>,
    pub release_point: URID<ReleasePoint>,
    pub idle_point: URID<IdlePoint>,
    pub audio_data: URID<AudioData>,
    pub input_signal: URID<InputSignal>,
    pub output_signal: URID<OutputSignal>,
    pub gain_signal: URID<GainSignal>
}
