use crate::error::{Error, Result};
use crate::wmi::WbemClassObjectExt;
use windows::Win32::System::Wmi::IWbemClassObject;

/// Represents a virtual network adapter attached to a VM.
#[derive(Debug)]
pub struct NetworkAdapter {
    /// Adapter instance ID.
    pub instance_id: String,
    /// Adapter name/element name.
    pub name: String,
    /// MAC address (if static).
    pub mac_address: Option<String>,
    /// Whether MAC is dynamic.
    pub dynamic_mac: bool,
    /// Connected switch name.
    pub switch_name: Option<String>,
    /// VLAN ID (if configured).
    pub vlan_id: Option<u16>,
    /// WMI path.
    path: String,
}

impl NetworkAdapter {
    /// Create from WMI object.
    pub(crate) fn from_wmi(obj: &IWbemClassObject) -> Result<Self> {
        let instance_id = obj.get_string_prop_required("InstanceID")?;
        let name = obj.get_string_prop("ElementName")?.unwrap_or_default();
        let path = obj.get_path()?;
        let mac_address = obj.get_string_prop("Address")?;
        let dynamic_mac = obj.get_bool("StaticMacAddress")?.map(|s| !s).unwrap_or(true);

        Ok(Self {
            instance_id,
            name,
            mac_address,
            dynamic_mac,
            switch_name: None,
            vlan_id: None,
            path,
        })
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}

/// Settings for creating a network adapter.
#[derive(Debug, Clone)]
pub struct NetworkAdapterSettings {
    /// Adapter name.
    pub name: Option<String>,
    /// Switch to connect to.
    pub switch_name: Option<String>,
    /// Static MAC address (if not dynamic).
    pub mac_address: Option<String>,
    /// Use dynamic MAC address.
    pub dynamic_mac: bool,
    /// VLAN ID.
    pub vlan_id: Option<u16>,
    /// Enable MAC address spoofing.
    pub mac_spoofing: bool,
    /// Enable DHCP guard.
    pub dhcp_guard: bool,
    /// Enable router guard.
    pub router_guard: bool,
    /// Enable port mirroring.
    pub port_mirroring: PortMirroringMode,
    /// Bandwidth management settings.
    pub bandwidth: Option<BandwidthSettings>,
}

impl Default for NetworkAdapterSettings {
    fn default() -> Self {
        Self {
            name: None,
            switch_name: None,
            mac_address: None,
            dynamic_mac: true,
            vlan_id: None,
            mac_spoofing: false,
            dhcp_guard: false,
            router_guard: false,
            port_mirroring: PortMirroringMode::None,
            bandwidth: None,
        }
    }
}

impl NetworkAdapterSettings {
    /// Create a new builder.
    pub fn builder() -> NetworkAdapterSettingsBuilder {
        NetworkAdapterSettingsBuilder::default()
    }

    /// Validate settings.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref mac) = self.mac_address {
            // Validate MAC address format (XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX)
            let mac_clean = mac.replace([':', '-'], "");
            if mac_clean.len() != 12 || !mac_clean.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(Error::Validation {
                    field: "mac_address",
                    message: "Invalid MAC address format".to_string(),
                });
            }
        }

        if let Some(vlan) = self.vlan_id {
            if vlan > 4094 {
                return Err(Error::Validation {
                    field: "vlan_id",
                    message: "VLAN ID must be 0-4094".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Builder for network adapter settings.
#[derive(Default)]
pub struct NetworkAdapterSettingsBuilder {
    name: Option<String>,
    switch_name: Option<String>,
    mac_address: Option<String>,
    dynamic_mac: bool,
    vlan_id: Option<u16>,
    mac_spoofing: bool,
    dhcp_guard: bool,
    router_guard: bool,
    port_mirroring: PortMirroringMode,
    bandwidth: Option<BandwidthSettings>,
}

impl NetworkAdapterSettingsBuilder {
    /// Create with default dynamic MAC.
    pub fn new() -> Self {
        Self {
            dynamic_mac: true,
            ..Default::default()
        }
    }

    /// Set adapter name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Connect to a virtual switch.
    pub fn switch(mut self, switch_name: impl Into<String>) -> Self {
        self.switch_name = Some(switch_name.into());
        self
    }

    /// Set static MAC address.
    pub fn mac_address(mut self, mac: impl Into<String>) -> Self {
        self.mac_address = Some(mac.into());
        self.dynamic_mac = false;
        self
    }

    /// Use dynamic MAC address.
    pub fn dynamic_mac(mut self, dynamic: bool) -> Self {
        self.dynamic_mac = dynamic;
        if dynamic {
            self.mac_address = None;
        }
        self
    }

    /// Set VLAN ID.
    pub fn vlan_id(mut self, vlan: u16) -> Self {
        self.vlan_id = Some(vlan);
        self
    }

    /// Enable MAC address spoofing.
    pub fn mac_spoofing(mut self, enabled: bool) -> Self {
        self.mac_spoofing = enabled;
        self
    }

    /// Enable DHCP guard.
    pub fn dhcp_guard(mut self, enabled: bool) -> Self {
        self.dhcp_guard = enabled;
        self
    }

    /// Enable router guard.
    pub fn router_guard(mut self, enabled: bool) -> Self {
        self.router_guard = enabled;
        self
    }

    /// Set port mirroring mode.
    pub fn port_mirroring(mut self, mode: PortMirroringMode) -> Self {
        self.port_mirroring = mode;
        self
    }

    /// Set bandwidth limits.
    pub fn bandwidth(mut self, settings: BandwidthSettings) -> Self {
        self.bandwidth = Some(settings);
        self
    }

    /// Build and validate settings.
    pub fn build(self) -> Result<NetworkAdapterSettings> {
        let settings = NetworkAdapterSettings {
            name: self.name,
            switch_name: self.switch_name,
            mac_address: self.mac_address,
            dynamic_mac: self.dynamic_mac,
            vlan_id: self.vlan_id,
            mac_spoofing: self.mac_spoofing,
            dhcp_guard: self.dhcp_guard,
            router_guard: self.router_guard,
            port_mirroring: self.port_mirroring,
            bandwidth: self.bandwidth,
        };

        settings.validate()?;
        Ok(settings)
    }
}

/// Port mirroring mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortMirroringMode {
    #[default]
    None,
    Source,
    Destination,
}

impl PortMirroringMode {
    pub fn to_value(&self) -> u32 {
        match self {
            PortMirroringMode::None => 0,
            PortMirroringMode::Source => 1,
            PortMirroringMode::Destination => 2,
        }
    }
}

/// Bandwidth management settings.
#[derive(Debug, Clone)]
pub struct BandwidthSettings {
    /// Minimum bandwidth in Mbps.
    pub minimum_mbps: Option<u64>,
    /// Maximum bandwidth in Mbps.
    pub maximum_mbps: Option<u64>,
    /// Burst size in MB.
    pub burst_mb: Option<u64>,
}

impl Default for BandwidthSettings {
    fn default() -> Self {
        Self {
            minimum_mbps: None,
            maximum_mbps: None,
            burst_mb: None,
        }
    }
}

impl BandwidthSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minimum_mbps(mut self, mbps: u64) -> Self {
        self.minimum_mbps = Some(mbps);
        self
    }

    pub fn maximum_mbps(mut self, mbps: u64) -> Self {
        self.maximum_mbps = Some(mbps);
        self
    }

    pub fn burst_mb(mut self, mb: u64) -> Self {
        self.burst_mb = Some(mb);
        self
    }
}
