use lv2::prelude::*;
use lv2_ui::*;

#[uri("http://johannes-mueller.org/lv2/envolvigo#uiOn")]
pub struct UIOn;

#[uri("http://johannes-mueller.org/lv2/envolvigo#uiOff")]
pub struct UIOff;

#[uri("http://johannes-mueller.org/lv2/envolvigo#TriggerState")]
pub struct TriggerState;

#[uri("http://johannes-mueller.org/lv2/envolvigo#triggerTime")]
pub struct TriggerTime;

#[uri("http://johannes-mueller.org/lv2/envolvigo#triggerLevel")]
pub struct TriggerLevel;

#[derive(URIDCollection)]
pub struct URIDs {
    pub atom: AtomURIDCollection,
    pub unit: UnitURIDCollection,
    pub ui_on: URID<UIOn>,
    pub ui_off: URID<UIOff>,
    pub trigger_state: URID<TriggerState>,
    pub trigger_time: URID<TriggerTime>,
    pub trigger_level: URID<TriggerLevel>,
    pub atom_event_transfer: URID<AtomEventTransfer>
}
