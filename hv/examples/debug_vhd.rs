//! Debug VHD creation via WMI

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hv::wmi::WmiConnection;
    use windows::core::BSTR;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::Wmi::{
        IWbemObjectTextSrc, WbemObjectTextSrc, WMI_OBJ_TEXT_WMI_DTD_2_0,
    };

    println!("Connecting to Hyper-V WMI...");
    let conn = WmiConnection::connect_hyperv()?;

    // Get ImageManagementService
    println!("Getting ImageManagementService...");
    let mut ims_result = conn.query("SELECT * FROM Msvm_ImageManagementService")?;
    let ims = ims_result.next().ok_or("IMS not found")??;
    let ims_path = ims.path()?;
    println!("IMS path: {}", ims_path);

    // Create VHD settings
    println!("\nCreating VHD settings...");
    let vhd_class = conn.get_class("Msvm_VirtualHardDiskSettingData")?;
    let vhd_settings = vhd_class.spawn_instance()?;

    let path = "C:\\Hyper-V\\Virtual Hard Disks\\test_debug.vhdx";
    let size: u64 = 10 * 1024 * 1024 * 1024; // 10GB

    println!("  Path: {}", path);
    println!("  Size: {} bytes", size);

    vhd_settings.put_string("Path", path)?;
    vhd_settings.put_u64("MaxInternalSize", size)?;
    vhd_settings.put_u16("Type", 3)?; // Dynamic
    vhd_settings.put_u16("Format", 3)?; // VHDX
    vhd_settings.put_u32("BlockSize", 0)?;
    vhd_settings.put_u32("LogicalSectorSize", 0)?;

    // Get object text using WMI DTD 2.0 format (required for embedded instances)
    println!("\nGetting object text (WMI_DTD_2_0 format)...");
    let text = unsafe {
        // Create the text source object
        let text_src: IWbemObjectTextSrc =
            CoCreateInstance(&WbemObjectTextSrc, None, CLSCTX_INPROC_SERVER)?;

        // Get text in WMI DTD 2.0 format
        text_src.GetText(
            0,
            vhd_settings.inner(),
            WMI_OBJ_TEXT_WMI_DTD_2_0.0 as u32,
            None,
        )?
    };
    println!("VHD Settings Text:\n{}", text.to_string());

    // Try to create
    println!("\nCalling CreateVirtualHardDisk...");
    let params = conn.get_method_params("Msvm_ImageManagementService", "CreateVirtualHardDisk")?;
    params.put_string("VirtualDiskSettingData", &text.to_string())?;

    let result = conn.exec_method(&ims_path, "CreateVirtualHardDisk", Some(&params))?;

    if let Some(result_obj) = result {
        let return_value = result_obj.get_u32("ReturnValue")?;
        println!("ReturnValue: {:?}", return_value);

        if return_value == Some(4096) {
            println!("Job started, checking job...");
            if let Some(job_path) = result_obj.get_string("Job")? {
                println!("Job path: {}", job_path);
                // Wait and check job
                std::thread::sleep(std::time::Duration::from_secs(2));
                let job = conn.get_object(&job_path)?;
                let job_state = job.get_u32("JobState")?;
                let error_code = job.get_u32("ErrorCode")?;
                let error_desc = job.get_string("ErrorDescription")?;
                println!("JobState: {:?}", job_state);
                println!("ErrorCode: {:?}", error_code);
                println!("ErrorDescription: {:?}", error_desc);
            }
        } else if return_value == Some(0) {
            println!("VHD created successfully!");
        } else {
            println!("Failed with code: {:?}", return_value);
            // Try to get error description from result
            let error_desc = result_obj.get_string("ErrorDescription")?;
            println!("ErrorDescription: {:?}", error_desc);
        }
    }

    // Also try the simple approach - check if the directory exists
    println!("\n--- Checking prerequisites ---");
    let dir = std::path::Path::new("C:\\Hyper-V\\Virtual Hard Disks");
    println!("Directory exists: {}", dir.exists());
    if !dir.exists() {
        println!("Creating directory...");
        std::fs::create_dir_all(dir)?;
        println!("Directory created, try running again!");
    }

    // Check if file already exists
    let file = std::path::Path::new(path);
    println!("File already exists: {}", file.exists());
    if file.exists() {
        println!("Removing existing file...");
        std::fs::remove_file(file)?;
        println!("File removed, try running again!");
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows");
}
