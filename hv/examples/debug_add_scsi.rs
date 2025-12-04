//! Debug adding SCSI controller to a VM

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hv::wmi::{hyperv, WmiConnection};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::Wmi::{
        IWbemObjectTextSrc, WbemObjectTextSrc, WMI_OBJ_TEXT_WMI_DTD_2_0,
    };

    println!("=== Debug Add SCSI Controller ===\n");

    let conn = WmiConnection::connect_hyperv()?;

    // First, we need a VM to test with. Let's create a minimal one.
    let test_vm_name = "DebugScsiTestVM";

    // Check if VM exists and delete it
    let check_query = format!(
        r#"SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine' AND ElementName = '{}'"#,
        test_vm_name
    );
    let mut existing = conn.query(&check_query)?;
    if existing.next().is_some() {
        println!("Deleting existing test VM...");
        let vsms = hyperv::get_vsms(&conn)?;
        let vsms_path = vsms.path()?;

        let vm_query = format!(
            r#"SELECT * FROM Msvm_ComputerSystem WHERE ElementName = '{}'"#,
            test_vm_name
        );
        let mut vms = conn.query(&vm_query)?;
        if let Some(vm) = vms.next() {
            let vm = vm?;
            let vm_path = vm.path()?;
            let params = conn.get_method_params(
                "Msvm_VirtualSystemManagementService",
                "DestroySystem",
            )?;
            params.put_string("AffectedSystem", &vm_path)?;
            conn.exec_method(&vsms_path, "DestroySystem", Some(&params))?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // Create a minimal VM
    println!("Step 1: Creating minimal test VM...");
    let vsms = hyperv::get_vsms(&conn)?;
    let vsms_path = vsms.path()?;

    let settings_class = conn.get_class("Msvm_VirtualSystemSettingData")?;
    let settings = settings_class.spawn_instance()?;
    settings.put_string("ElementName", test_vm_name)?;
    settings.put_string("VirtualSystemSubType", "Microsoft:Hyper-V:SubType:2")?; // Gen2

    // Get settings text
    let settings_text = unsafe {
        let text_src: IWbemObjectTextSrc =
            CoCreateInstance(&WbemObjectTextSrc, None, CLSCTX_INPROC_SERVER)?;
        text_src
            .GetText(0, settings.inner(), WMI_OBJ_TEXT_WMI_DTD_2_0.0 as u32, None)?
            .to_string()
    };
    println!("VM Settings XML:\n{}\n", settings_text);

    let define_params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "DefineSystem")?;
    define_params.put_string("SystemSettings", &settings_text)?;

    let result = conn.exec_method(&vsms_path, "DefineSystem", Some(&define_params))?;

    let vm_path = if let Some(result_obj) = result {
        let return_value = result_obj.get_u32("ReturnValue")?.unwrap_or(999);
        println!("DefineSystem ReturnValue: {}", return_value);

        if return_value == 4096 {
            let job_path = result_obj.get_string("Job")?.unwrap_or_default();
            println!("Waiting for job: {}", job_path);
            hyperv::wait_for_job(&conn, &job_path)?;
        } else if return_value != 0 {
            println!("ERROR: DefineSystem failed with code {}", return_value);
            return Ok(());
        }

        result_obj.get_string("ResultingSystem")?.unwrap_or_default()
    } else {
        println!("ERROR: No result from DefineSystem");
        return Ok(());
    };

    println!("VM created: {}", vm_path);

    // Get VM settings path
    let vm_obj = conn.get_object(&vm_path)?;
    let vm_id = vm_obj.get_string("Name")?.unwrap_or_default();
    println!("VM ID: {}", vm_id);

    let settings_query = format!(
        r#"SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier = '{}' AND VirtualSystemType = 'Microsoft:Hyper-V:System:Realized'"#,
        vm_id
    );
    let mut settings_results = conn.query(&settings_query)?;
    let vm_settings = settings_results.next().ok_or("VM settings not found")??;
    let vm_settings_path = vm_settings.path()?;
    println!("VM Settings Path: {}", vm_settings_path);

    // Step 2: Get default SCSI controller
    println!("\nStep 2: Getting default SCSI controller resource...");

    let pool_query = r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = 'Microsoft:Hyper-V:Synthetic SCSI Controller' AND Primordial = TRUE"#;
    let mut pools = conn.query(pool_query)?;
    let pool = pools.next().ok_or("Pool not found")??;
    let pool_path = pool.path()?;

    let caps_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
        pool_path
    );
    let mut caps = conn.query(&caps_query)?;
    let cap = caps.next().ok_or("Capabilities not found")??;
    let cap_path = cap.path()?;

    let assoc_query = format!(
        r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
        cap_path
    );
    let assocs = conn.query(&assoc_query)?;

    let mut scsi_default = None;
    for assoc in assocs {
        let assoc = assoc?;
        if assoc.get_u32("ValueRole")? == Some(0) {
            if let Some(part_path) = assoc.get_string("PartComponent")? {
                scsi_default = Some(conn.get_object(&part_path)?);
                break;
            }
        }
    }

    let scsi = scsi_default.ok_or("Default SCSI not found")?;
    println!("Got default SCSI controller");
    println!("  ResourceType: {:?}", scsi.get_u32("ResourceType")?);
    println!("  ResourceSubType: {:?}", scsi.get_string("ResourceSubType")?);

    // Step 3: Serialize to XML
    println!("\nStep 3: Serializing SCSI controller to XML...");
    let scsi_text = unsafe {
        let text_src: IWbemObjectTextSrc =
            CoCreateInstance(&WbemObjectTextSrc, None, CLSCTX_INPROC_SERVER)?;
        text_src
            .GetText(0, scsi.inner(), WMI_OBJ_TEXT_WMI_DTD_2_0.0 as u32, None)?
            .to_string()
    };
    println!("SCSI Controller XML:\n{}\n", scsi_text);

    // Step 4: Call AddResourceSettings
    println!("Step 4: Calling AddResourceSettings...");

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", &vm_settings_path)?;

    // Create SAFEARRAY for ResourceSettings
    use windows::Win32::System::Com::SAFEARRAYBOUND;
    use windows::Win32::System::Ole::{SafeArrayCreate, SafeArrayPutElement};
    use windows::Win32::System::Variant::{VT_ARRAY, VT_BSTR};
    use windows::core::BSTR;

    unsafe {
        let bounds = SAFEARRAYBOUND {
            cElements: 1,
            lLbound: 0,
        };
        let sa = SafeArrayCreate(VT_BSTR, 1, &bounds);
        let bstr = BSTR::from(&scsi_text);
        let index: i32 = 0;
        SafeArrayPutElement(sa, &index, bstr.into_raw() as *const _)?;

        // Set on params
        use windows::Win32::System::Variant::VARIANT;
        use windows::core::PCWSTR;

        let prop_name: Vec<u16> = "ResourceSettings\0".encode_utf16().collect();
        let mut variant = VARIANT::default();
        (*variant.Anonymous.Anonymous).vt = VT_ARRAY | VT_BSTR;
        (*variant.Anonymous.Anonymous).Anonymous.parray = sa;

        params
            .inner()
            .Put(PCWSTR::from_raw(prop_name.as_ptr()), 0, &variant, 0)?;
    }

    println!("Executing AddResourceSettings...");
    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        let return_value = result_obj.get_u32("ReturnValue")?.unwrap_or(999);
        println!("ReturnValue: {}", return_value);

        if return_value == 4096 {
            let job_path = result_obj.get_string("Job")?.unwrap_or_default();
            println!("Job started: {}", job_path);
            std::thread::sleep(std::time::Duration::from_secs(2));
            let job = conn.get_object(&job_path)?;
            println!("JobState: {:?}", job.get_u32("JobState")?);
            println!("ErrorCode: {:?}", job.get_u32("ErrorCode")?);
            println!("ErrorDescription: {:?}", job.get_string("ErrorDescription")?);
        } else if return_value == 0 {
            println!("SUCCESS! SCSI controller added.");
            let resulting = result_obj.get_string("ResultingResourceSettings")?;
            println!("ResultingResourceSettings: {:?}", resulting);
        } else {
            println!("FAILED with return code: {}", return_value);
        }
    }

    // Cleanup: Delete test VM
    println!("\nCleaning up test VM...");
    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "DestroySystem")?;
    params.put_string("AffectedSystem", &vm_path)?;
    conn.exec_method(&vsms_path, "DestroySystem", Some(&params))?;
    println!("Test VM deleted.");

    println!("\n=== Done ===");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows");
}
