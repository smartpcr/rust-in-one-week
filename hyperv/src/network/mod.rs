mod adapter;
mod switch;

pub use adapter::{
    BandwidthSettings, NetworkAdapter, NetworkAdapterSettings, NetworkAdapterSettingsBuilder,
    PortMirroringMode,
};
pub use switch::{SwitchType, VirtualSwitch, VirtualSwitchSettings, VirtualSwitchSettingsBuilder};
