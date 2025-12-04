//! Debug script to test querying default resources from Hyper-V

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hv::wmi::WmiConnection;

    println!("Connecting to Hyper-V WMI...");
    let conn = WmiConnection::connect_hyperv()?;

    // Test querying resource pools
    println!("\n=== Querying Resource Pools ===");
    let pools = conn.query("SELECT * FROM Msvm_ResourcePool WHERE Primordial = TRUE")?;
    for pool in pools {
        let pool = pool?;
        if let Some(subtype) = pool.get_string("ResourceSubType")? {
            println!("Pool: {}", subtype);
        }
    }

    // Test querying SCSI controller default
    println!("\n=== Testing SCSI Controller Default ===");
    let scsi_pool_query = r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = 'Microsoft:Hyper-V:Synthetic SCSI Controller' AND Primordial = TRUE"#;
    let mut pools = conn.query(scsi_pool_query)?;
    if let Some(pool) = pools.next() {
        let pool = pool?;
        let pool_path = pool.path()?;
        println!("SCSI Pool Path: {}", pool_path);

        // Get allocation capabilities
        let caps_query = format!(
            r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
            pool_path
        );
        let mut caps = conn.query(&caps_query)?;
        if let Some(cap) = caps.next() {
            let cap = cap?;
            let cap_path = cap.path()?;
            println!("Capabilities Path: {}", cap_path);

            // Get associations with ValueRole
            let assoc_query = format!(
                r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
                cap_path
            );
            let assocs = conn.query(&assoc_query)?;
            for assoc in assocs {
                let assoc = assoc?;
                let role = assoc.get_u32("ValueRole")?.unwrap_or(999);
                let part = assoc.get_string("PartComponent")?;
                println!("  ValueRole: {} -> {:?}", role, part);

                if role == 0 {
                    if let Some(part_path) = part {
                        println!("\n  Getting default setting...");
                        let default_obj = conn.get_object(&part_path)?;
                        println!(
                            "  Default ResourceType: {:?}",
                            default_obj.get_u32("ResourceType")?
                        );
                        println!(
                            "  Default ResourceSubType: {:?}",
                            default_obj.get_string("ResourceSubType")?
                        );
                        println!(
                            "  Default InstanceID: {:?}",
                            default_obj.get_string("InstanceID")?
                        );
                    }
                }
            }
        }
    }

    println!("\n=== Testing Synthetic Disk Drive Default ===");
    let disk_pool_query = r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = 'Microsoft:Hyper-V:Synthetic Disk Drive' AND Primordial = TRUE"#;
    let mut pools = conn.query(disk_pool_query)?;
    if let Some(pool) = pools.next() {
        let pool = pool?;
        let pool_path = pool.path()?;
        println!("Disk Drive Pool Path: {}", pool_path);

        let caps_query = format!(
            r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
            pool_path
        );
        let mut caps = conn.query(&caps_query)?;
        if let Some(cap) = caps.next() {
            let cap = cap?;
            let cap_path = cap.path()?;

            let assoc_query = format!(
                r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
                cap_path
            );
            let assocs = conn.query(&assoc_query)?;
            for assoc in assocs {
                let assoc = assoc?;
                let role = assoc.get_u32("ValueRole")?.unwrap_or(999);
                if role == 0 {
                    if let Some(part_path) = assoc.get_string("PartComponent")? {
                        let default_obj = conn.get_object(&part_path)?;
                        println!("Default found!");
                        println!("  ResourceType: {:?}", default_obj.get_u32("ResourceType")?);
                        println!(
                            "  ResourceSubType: {:?}",
                            default_obj.get_string("ResourceSubType")?
                        );
                    }
                }
            }
        }
    }

    println!("\n=== Testing Virtual Hard Disk Default ===");
    let vhd_pool_query = r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = 'Microsoft:Hyper-V:Virtual Hard Disk' AND Primordial = TRUE"#;
    let mut pools = conn.query(vhd_pool_query)?;
    if let Some(pool) = pools.next() {
        let pool = pool?;
        let pool_path = pool.path()?;
        println!("VHD Pool Path: {}", pool_path);

        let caps_query = format!(
            r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
            pool_path
        );
        let mut caps = conn.query(&caps_query)?;
        if let Some(cap) = caps.next() {
            let cap = cap?;
            let cap_path = cap.path()?;

            let assoc_query = format!(
                r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
                cap_path
            );
            let assocs = conn.query(&assoc_query)?;
            for assoc in assocs {
                let assoc = assoc?;
                let role = assoc.get_u32("ValueRole")?.unwrap_or(999);
                if role == 0 {
                    if let Some(part_path) = assoc.get_string("PartComponent")? {
                        let default_obj = conn.get_object(&part_path)?;
                        println!("Default found!");
                        println!("  ResourceType: {:?}", default_obj.get_u32("ResourceType")?);
                        println!(
                            "  ResourceSubType: {:?}",
                            default_obj.get_string("ResourceSubType")?
                        );
                    }
                }
            }
        }
    }

    println!("\nDone!");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows");
}
