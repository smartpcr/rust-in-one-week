//! Processor settings management for Hyper-V VMs.

use super::topology::NumaTopology;
use super::types::*;
use crate::error::{Error, Result};

#[cfg(windows)]
use crate::wmi::{WbemClassObjectExt, WmiConnection};

/// Advanced processor settings for a VM.
#[derive(Debug, Clone)]
pub struct ProcessorSettings {
    /// Number of virtual processors.
    pub count: u32,
    /// CPU limit (percentage).
    pub limit: CpuLimit,
    /// CPU reservation (percentage).
    pub reservation: CpuReservation,
    /// Relative weight for CPU scheduling.
    pub weight: CpuWeight,
    /// Hardware threads per core (SMT configuration).
    pub hw_threads_per_core: Option<HwThreadsPerCore>,
    /// Expose virtualization extensions (nested virtualization).
    pub expose_virtualization_extensions: bool,
    /// Enable hierarchical virtualization.
    pub enable_hierarchical_virtualization: bool,
    /// Maximum hierarchical partitions.
    pub max_hierarchical_partitions: Option<u32>,
    /// Maximum hierarchical VPs.
    pub max_hierarchical_vps: Option<u32>,
    /// CPU group ID.
    pub cpu_group_id: Option<String>,
    /// L3 cache ways (AMD).
    pub l3_cache_ways: Option<u32>,
    /// L3 processor distribution policy (AMD).
    pub l3_distribution_policy: L3DistributionPolicy,
    /// Enable host resource protection.
    pub enable_host_resource_protection: bool,
    /// Enable processor page shattering mitigation.
    pub enable_page_shattering_mitigation: bool,
    /// NUMA topology.
    pub numa_topology: Option<NumaTopology>,
}

impl Default for ProcessorSettings {
    fn default() -> Self {
        Self {
            count: 1,
            limit: CpuLimit::NONE,
            reservation: CpuReservation::NONE,
            weight: CpuWeight::DEFAULT,
            hw_threads_per_core: None,
            expose_virtualization_extensions: false,
            enable_hierarchical_virtualization: false,
            max_hierarchical_partitions: None,
            max_hierarchical_vps: None,
            cpu_group_id: None,
            l3_cache_ways: None,
            l3_distribution_policy: L3DistributionPolicy::Default,
            enable_host_resource_protection: false,
            enable_page_shattering_mitigation: false,
            numa_topology: None,
        }
    }
}

impl ProcessorSettings {
    /// Create a new builder.
    pub fn builder() -> ProcessorSettingsBuilder {
        ProcessorSettingsBuilder::new()
    }

    /// Create settings for basic configuration.
    pub fn basic(count: u32) -> Self {
        Self {
            count,
            ..Default::default()
        }
    }

    /// Create settings for nested virtualization.
    pub fn nested_virtualization(count: u32) -> Self {
        Self {
            count,
            expose_virtualization_extensions: true,
            ..Default::default()
        }
    }

    /// Get processor settings for a VM.
    #[cfg(windows)]
    pub fn get(conn: &WmiConnection, vm_id: &str) -> Result<Self> {
        // Query Msvm_ProcessorSettingData via association
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState \
             ResultClass=Msvm_VirtualSystemSettingData",
            vm_id.replace('\'', "''")
        );

        let vssd_results = conn.query(&query)?;
        if vssd_results.is_empty() {
            return Ok(ProcessorSettings::default());
        }

        let vssd = &vssd_results[0];
        let vssd_path = vssd.get_path()?;

        // Query ProcessorSettingData from VSSD
        let proc_query = format!(
            "ASSOCIATORS OF {{{}}} \
             WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
             ResultClass=Msvm_ProcessorSettingData",
            vssd_path
        );

        let proc_results = conn.query(&proc_query)?;

        if let Some(obj) = proc_results.first() {
            let count = obj.get_u32("VirtualQuantity")?.unwrap_or(1);
            let limit_raw = obj.get_u64("Limit")?.unwrap_or(100000);
            let reservation_raw = obj.get_u64("Reservation")?.unwrap_or(0);
            let weight_raw = obj.get_u32("Weight")?.unwrap_or(100);

            let hw_threads = obj.get_u32("HwThreadsPerCore")?;
            let expose_virt = obj
                .get_bool("ExposeVirtualizationExtensions")?
                .unwrap_or(false);
            let cpu_group = obj.get_string_prop("CpuGroupId")?;
            let l3_ways = obj.get_u32("L3CacheWays")?;

            Ok(ProcessorSettings {
                count,
                limit: CpuLimit::from_raw(limit_raw).unwrap_or(CpuLimit::NONE),
                reservation: CpuReservation::from_raw(reservation_raw)
                    .unwrap_or(CpuReservation::NONE),
                weight: CpuWeight::new(weight_raw).unwrap_or(CpuWeight::DEFAULT),
                hw_threads_per_core: hw_threads.and_then(|v| HwThreadsPerCore::new(v).ok()),
                expose_virtualization_extensions: expose_virt,
                enable_hierarchical_virtualization: false,
                max_hierarchical_partitions: None,
                max_hierarchical_vps: None,
                cpu_group_id: cpu_group,
                l3_cache_ways: l3_ways,
                l3_distribution_policy: L3DistributionPolicy::Default,
                enable_host_resource_protection: false,
                enable_page_shattering_mitigation: false,
                numa_topology: None,
            })
        } else {
            Ok(ProcessorSettings::default())
        }
    }

    /// Apply processor settings to a VM.
    #[cfg(windows)]
    pub fn apply(&self, conn: &WmiConnection, vm_id: &str) -> Result<()> {
        use std::time::Duration;

        // Get VSMS
        let vsms = conn.get_singleton("Msvm_VirtualSystemManagementService")?;
        let vsms_path = vsms.get_path()?;

        // Get current VSSD
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState \
             ResultClass=Msvm_VirtualSystemSettingData",
            vm_id.replace('\'', "''")
        );

        let vssd_results = conn.query(&query)?;
        if vssd_results.is_empty() {
            return Err(Error::VmNotFound(vm_id.to_string()));
        }

        let vssd = &vssd_results[0];
        let vssd_path = vssd.get_path()?;

        // Query ProcessorSettingData from VSSD
        let proc_query = format!(
            "ASSOCIATORS OF {{{}}} \
             WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent \
             ResultClass=Msvm_ProcessorSettingData",
            vssd_path
        );

        let proc_results = conn.query(&proc_query)?;

        if let Some(proc_obj) = proc_results.first() {
            // Modify processor settings
            proc_obj.put_u32("VirtualQuantity", self.count)?;
            proc_obj.put_u64("Limit", self.limit.raw())?;
            proc_obj.put_u64("Reservation", self.reservation.raw())?;
            proc_obj.put_u32("Weight", self.weight.value())?;

            if let Some(ref hw_threads) = self.hw_threads_per_core {
                proc_obj.put_u32("HwThreadsPerCore", hw_threads.value())?;
            }

            proc_obj.put_bool("ExposeVirtualizationExtensions", self.expose_virtualization_extensions)?;

            if let Some(ref cpu_group) = self.cpu_group_id {
                proc_obj.put_string("CpuGroupId", cpu_group)?;
            }

            if let Some(l3_ways) = self.l3_cache_ways {
                proc_obj.put_u32("L3CacheWays", l3_ways)?;
            }

            let proc_text = proc_obj.get_text()?;

            // Call ModifyResourceSettings
            let in_params = conn.get_method_params(
                "Msvm_VirtualSystemManagementService",
                "ModifyResourceSettings",
            )?;
            in_params.put_string_array("ResourceSettings", &[&proc_text])?;

            let out_params =
                conn.exec_method(&vsms_path, "ModifyResourceSettings", Some(&in_params))?;

            // Check result
            let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
            match return_value {
                0 => Ok(()),
                4096 => {
                    let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                        Error::operation_failed(
                            "ModifyResourceSettings",
                            4096,
                            "No job path returned",
                        )
                    })?;

                    let waiter = crate::wmi::JobWaiter::with_timeout(conn, Duration::from_secs(60));
                    waiter.wait_for_job(&job_path, "ModifyResourceSettings")?;
                    Ok(())
                }
                code => Err(Error::operation_failed(
                    "ModifyResourceSettings",
                    code,
                    "Failed to modify processor settings",
                )),
            }
        } else {
            Err(Error::operation_failed(
                "ModifyResourceSettings",
                0,
                "Processor settings not found for VM",
            ))
        }
    }
}

/// Builder for processor settings.
#[derive(Debug, Clone, Default)]
pub struct ProcessorSettingsBuilder {
    settings: ProcessorSettings,
}

impl ProcessorSettingsBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of virtual processors.
    pub fn count(mut self, count: u32) -> Self {
        self.settings.count = count;
        self
    }

    /// Set CPU limit.
    pub fn limit(mut self, limit: CpuLimit) -> Self {
        self.settings.limit = limit;
        self
    }

    /// Set CPU limit from percentage.
    pub fn limit_percent(mut self, percent: f64) -> Result<Self> {
        self.settings.limit = CpuLimit::from_percent(percent).ok_or_else(|| Error::Validation {
            field: "limit",
            message: format!("Invalid CPU limit percentage: {}", percent),
        })?;
        Ok(self)
    }

    /// Set CPU reservation.
    pub fn reservation(mut self, reservation: CpuReservation) -> Self {
        self.settings.reservation = reservation;
        self
    }

    /// Set CPU reservation from percentage.
    pub fn reservation_percent(mut self, percent: f64) -> Result<Self> {
        self.settings.reservation =
            CpuReservation::from_percent(percent).ok_or_else(|| Error::Validation {
                field: "reservation",
                message: format!("Invalid CPU reservation percentage: {}", percent),
            })?;
        Ok(self)
    }

    /// Set CPU weight.
    pub fn weight(mut self, weight: CpuWeight) -> Self {
        self.settings.weight = weight;
        self
    }

    /// Set hardware threads per core.
    pub fn hw_threads_per_core(mut self, threads: HwThreadsPerCore) -> Self {
        self.settings.hw_threads_per_core = Some(threads);
        self
    }

    /// Set hardware threads per core from value.
    pub fn hw_threads_per_core_value(mut self, threads: u32) -> Result<Self> {
        self.settings.hw_threads_per_core = Some(HwThreadsPerCore::new(threads)?);
        Ok(self)
    }

    /// Enable or disable nested virtualization.
    pub fn expose_virtualization_extensions(mut self, enabled: bool) -> Self {
        self.settings.expose_virtualization_extensions = enabled;
        self
    }

    /// Enable nested virtualization.
    pub fn nested_virtualization(self) -> Self {
        self.expose_virtualization_extensions(true)
    }

    /// Set CPU group ID.
    pub fn cpu_group(mut self, group_id: impl Into<String>) -> Self {
        self.settings.cpu_group_id = Some(group_id.into());
        self
    }

    /// Set L3 cache ways (AMD).
    pub fn l3_cache_ways(mut self, ways: u32) -> Self {
        self.settings.l3_cache_ways = Some(ways);
        self
    }

    /// Set L3 distribution policy (AMD).
    pub fn l3_distribution_policy(mut self, policy: L3DistributionPolicy) -> Self {
        self.settings.l3_distribution_policy = policy;
        self
    }

    /// Enable host resource protection.
    pub fn host_resource_protection(mut self, enabled: bool) -> Self {
        self.settings.enable_host_resource_protection = enabled;
        self
    }

    /// Enable page shattering mitigation.
    pub fn page_shattering_mitigation(mut self, enabled: bool) -> Self {
        self.settings.enable_page_shattering_mitigation = enabled;
        self
    }

    /// Set NUMA topology.
    pub fn numa_topology(mut self, topology: NumaTopology) -> Self {
        self.settings.numa_topology = Some(topology);
        self
    }

    /// Build the settings, validating the configuration.
    pub fn build(self) -> Result<ProcessorSettings> {
        // Validate processor count
        if self.settings.count == 0 {
            return Err(Error::Validation {
                field: "count",
                message: "Processor count must be at least 1".to_string(),
            });
        }

        if self.settings.count > 2048 {
            return Err(Error::Validation {
                field: "count",
                message: format!(
                    "Processor count {} exceeds maximum of 2048",
                    self.settings.count
                ),
            });
        }

        // Validate reservation doesn't exceed limit
        if self.settings.reservation.raw() > self.settings.limit.raw() {
            return Err(Error::Validation {
                field: "reservation",
                message: format!(
                    "CPU reservation ({}) cannot exceed limit ({})",
                    self.settings.reservation, self.settings.limit
                ),
            });
        }

        // Validate hierarchical virtualization settings
        if self.settings.enable_hierarchical_virtualization
            && !self.settings.expose_virtualization_extensions
        {
            return Err(Error::Validation {
                field: "enable_hierarchical_virtualization",
                message: "Hierarchical virtualization requires virtualization extensions to be exposed".to_string(),
            });
        }

        // Validate NUMA topology if specified
        if let Some(ref numa) = self.settings.numa_topology {
            let numa_procs = numa.total_processors();
            if numa_procs != self.settings.count {
                return Err(Error::Validation {
                    field: "numa_topology",
                    message: format!(
                        "NUMA topology total processors ({}) doesn't match processor count ({})",
                        numa_procs, self.settings.count
                    ),
                });
            }
        }

        Ok(self.settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = ProcessorSettings::default();
        assert_eq!(settings.count, 1);
        assert_eq!(settings.limit, CpuLimit::NONE);
        assert_eq!(settings.reservation, CpuReservation::NONE);
        assert_eq!(settings.weight, CpuWeight::DEFAULT);
        assert!(!settings.expose_virtualization_extensions);
    }

    #[test]
    fn test_basic_settings() {
        let settings = ProcessorSettings::basic(4);
        assert_eq!(settings.count, 4);
    }

    #[test]
    fn test_nested_virtualization_settings() {
        let settings = ProcessorSettings::nested_virtualization(4);
        assert_eq!(settings.count, 4);
        assert!(settings.expose_virtualization_extensions);
    }

    #[test]
    fn test_builder_basic() {
        let settings = ProcessorSettings::builder()
            .count(4)
            .limit(CpuLimit::from_percent(50.0).unwrap())
            .reservation(CpuReservation::from_percent(10.0).unwrap())
            .weight(CpuWeight::HIGH)
            .build()
            .unwrap();

        assert_eq!(settings.count, 4);
        assert_eq!(settings.limit.as_percent(), 50.0);
        assert_eq!(settings.reservation.as_percent(), 10.0);
        assert_eq!(settings.weight, CpuWeight::HIGH);
    }

    #[test]
    fn test_builder_percent_methods() {
        let settings = ProcessorSettings::builder()
            .count(2)
            .limit_percent(75.0)
            .unwrap()
            .reservation_percent(25.0)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(settings.limit.as_percent(), 75.0);
        assert_eq!(settings.reservation.as_percent(), 25.0);
    }

    #[test]
    fn test_builder_validation_zero_count() {
        let result = ProcessorSettings::builder().count(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validation_count_exceeds_max() {
        let result = ProcessorSettings::builder().count(3000).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validation_reservation_exceeds_limit() {
        let result = ProcessorSettings::builder()
            .count(2)
            .limit(CpuLimit::from_percent(50.0).unwrap())
            .reservation(CpuReservation::from_percent(75.0).unwrap())
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("reservation"));
    }

    #[test]
    fn test_builder_nested_virtualization() {
        let settings = ProcessorSettings::builder()
            .count(4)
            .nested_virtualization()
            .build()
            .unwrap();

        assert!(settings.expose_virtualization_extensions);
    }

    #[test]
    fn test_builder_hw_threads() {
        let settings = ProcessorSettings::builder()
            .count(4)
            .hw_threads_per_core(HwThreadsPerCore::TWO)
            .build()
            .unwrap();

        assert_eq!(settings.hw_threads_per_core, Some(HwThreadsPerCore::TWO));
    }

    #[test]
    fn test_builder_cpu_group() {
        let settings = ProcessorSettings::builder()
            .count(2)
            .cpu_group("my-cpu-group")
            .build()
            .unwrap();

        assert_eq!(settings.cpu_group_id, Some("my-cpu-group".to_string()));
    }

    #[test]
    fn test_builder_numa_topology_validation() {
        let topo = NumaTopology::symmetric(2, 4, 4096);

        // This should fail - processor count doesn't match
        let result = ProcessorSettings::builder()
            .count(4) // NUMA has 8 total
            .numa_topology(topo.clone())
            .build();
        assert!(result.is_err());

        // This should succeed
        let result = ProcessorSettings::builder()
            .count(8)
            .numa_topology(topo)
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_amd_settings() {
        let settings = ProcessorSettings::builder()
            .count(8)
            .l3_cache_ways(16)
            .l3_distribution_policy(L3DistributionPolicy::Localized)
            .build()
            .unwrap();

        assert_eq!(settings.l3_cache_ways, Some(16));
        assert_eq!(
            settings.l3_distribution_policy,
            L3DistributionPolicy::Localized
        );
    }
}
