mod switch;
mod adapter;

pub use switch::{VirtualSwitch, SwitchType, VirtualSwitchSettings, VirtualSwitchSettingsBuilder};
pub use adapter::{NetworkAdapter, NetworkAdapterSettings, NetworkAdapterSettingsBuilder, PortMirroringMode, BandwidthSettings};
