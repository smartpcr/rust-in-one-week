//! Debug SCSI controller creation

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hv::wmi::WmiConnection;

    println!("=== Debug SCSI Controller Resource Query ===\n");

    let conn = WmiConnection::connect_hyperv()?;

    // Step 1: Find SCSI controller resource pool
    println!("Step 1: Query SCSI Controller Resource Pool");
    let pool_query = r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = 'Microsoft:Hyper-V:Synthetic SCSI Controller' AND Primordial = TRUE"#;
    println!("Query: {}", pool_query);

    let mut pools = conn.query(pool_query)?;
    let pool = match pools.next() {
        Some(Ok(p)) => p,
        Some(Err(e)) => {
            println!("ERROR getting pool: {:?}", e);
            return Err(e.into());
        }
        None => {
            println!("ERROR: No SCSI controller pool found!");

            // Debug: list all resource pools
            println!("\nListing all resource pools:");
            let all_pools =
                conn.query("SELECT * FROM Msvm_ResourcePool WHERE Primordial = TRUE")?;
            for pool in all_pools {
                let pool = pool?;
                if let Some(subtype) = pool.get_string("ResourceSubType")? {
                    println!("  - {}", subtype);
                }
            }
            return Ok(());
        }
    };

    let pool_path = pool.path()?;
    println!("Pool found: {}", pool_path);

    // Step 2: Get allocation capabilities
    println!("\nStep 2: Query Allocation Capabilities");
    let caps_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
        pool_path
    );
    println!("Query: {}", caps_query);

    let mut caps_results = conn.query(&caps_query)?;
    let caps = match caps_results.next() {
        Some(Ok(c)) => c,
        Some(Err(e)) => {
            println!("ERROR getting capabilities: {:?}", e);
            return Err(e.into());
        }
        None => {
            println!("ERROR: No allocation capabilities found!");
            return Ok(());
        }
    };

    let caps_path = caps.path()?;
    println!("Capabilities found: {}", caps_path);

    // Step 3: Get SettingsDefineCapabilities associations
    println!("\nStep 3: Query SettingsDefineCapabilities (REFERENCES OF)");
    let assoc_query = format!(
        r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
        caps_path
    );
    println!("Query: {}", assoc_query);

    let assoc_results = conn.query(&assoc_query)?;
    let mut found_default = false;

    for assoc_result in assoc_results {
        let assoc = assoc_result?;
        let role = assoc.get_u32("ValueRole")?;
        let part = assoc.get_string("PartComponent")?;

        println!("\n  Association:");
        println!("    ValueRole: {:?}", role);
        println!("    PartComponent: {:?}", part);

        if role == Some(0) {
            found_default = true;
            if let Some(part_path) = part {
                println!("\n  Found DEFAULT (ValueRole=0)! Getting object...");
                match conn.get_object(&part_path) {
                    Ok(default_obj) => {
                        println!("  SUCCESS! Default object retrieved:");
                        println!(
                            "    ResourceType: {:?}",
                            default_obj.get_u32("ResourceType")?
                        );
                        println!(
                            "    ResourceSubType: {:?}",
                            default_obj.get_string("ResourceSubType")?
                        );
                        println!(
                            "    InstanceID: {:?}",
                            default_obj.get_string("InstanceID")?
                        );
                        println!(
                            "    ElementName: {:?}",
                            default_obj.get_string("ElementName")?
                        );
                    }
                    Err(e) => {
                        println!("  ERROR getting default object: {:?}", e);
                    }
                }
            }
        }
    }

    if !found_default {
        println!("\nERROR: No association with ValueRole=0 found!");
    }

    println!("\n=== Done ===");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows");
}
