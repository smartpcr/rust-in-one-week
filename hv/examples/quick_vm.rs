//! Example: Quick VM creation from OS VHDX templates
//!
//! Creates Windows or Linux VMs from pre-built VHDX template files.
//! This is the fastest way to create VMs when you have base images.

#[cfg(windows)]
use hv::{HyperV, VhdType, VmGeneration};
#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let hyperv = HyperV::new()?;
    let command = args[1].as_str();

    match command {
        // =================================================================
        // Windows VM from Template
        // =================================================================
        "windows" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;

            let memory_mb: u64 = args
                .get(4)
                .map(|s| s.parse().unwrap_or(4096))
                .unwrap_or(4096);
            let cpu_count: u32 = args.get(5).map(|s| s.parse().unwrap_or(2)).unwrap_or(2);

            create_windows_vm(&hyperv, vm_name, template_path, memory_mb, cpu_count)?;
        }

        // =================================================================
        // Linux VM from Template
        // =================================================================
        "linux" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;

            let memory_mb: u64 = args
                .get(4)
                .map(|s| s.parse().unwrap_or(2048))
                .unwrap_or(2048);
            let cpu_count: u32 = args.get(5).map(|s| s.parse().unwrap_or(2)).unwrap_or(2);

            create_linux_vm(&hyperv, vm_name, template_path, memory_mb, cpu_count)?;
        }

        // =================================================================
        // Windows VM with Differencing Disk (fast clone)
        // =================================================================
        "windows-clone" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let parent_vhdx = args.get(3).ok_or("Missing parent VHDX path")?;

            let memory_mb: u64 = args
                .get(4)
                .map(|s| s.parse().unwrap_or(4096))
                .unwrap_or(4096);
            let cpu_count: u32 = args.get(5).map(|s| s.parse().unwrap_or(2)).unwrap_or(2);

            create_windows_clone(&hyperv, vm_name, parent_vhdx, memory_mb, cpu_count)?;
        }

        // =================================================================
        // Linux VM with Differencing Disk (fast clone)
        // =================================================================
        "linux-clone" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let parent_vhdx = args.get(3).ok_or("Missing parent VHDX path")?;

            let memory_mb: u64 = args
                .get(4)
                .map(|s| s.parse().unwrap_or(2048))
                .unwrap_or(2048);
            let cpu_count: u32 = args.get(5).map(|s| s.parse().unwrap_or(2)).unwrap_or(2);

            create_linux_clone(&hyperv, vm_name, parent_vhdx, memory_mb, cpu_count)?;
        }

        // =================================================================
        // Dev Environment VM (Windows with more resources)
        // =================================================================
        "dev-windows" => {
            let vm_name = args.get(2).unwrap_or(&"DevVM".to_string()).clone();
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;

            create_dev_windows_vm(&hyperv, &vm_name, template_path)?;
        }

        // =================================================================
        // Dev Environment VM (Linux with more resources)
        // =================================================================
        "dev-linux" => {
            let vm_name = args.get(2).unwrap_or(&"DevLinux".to_string()).clone();
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;

            create_dev_linux_vm(&hyperv, &vm_name, template_path)?;
        }

        // =================================================================
        // Server VM (high resources)
        // =================================================================
        "server" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;
            let os_type = args.get(4).map(|s| s.as_str()).unwrap_or("windows");

            create_server_vm(&hyperv, vm_name, template_path, os_type)?;
        }

        // =================================================================
        // Batch create multiple VMs
        // =================================================================
        "batch" => {
            let prefix = args.get(2).ok_or("Missing VM name prefix")?;
            let template_path = args.get(3).ok_or("Missing template VHDX path")?;
            let count: u32 = args.get(4).map(|s| s.parse().unwrap_or(3)).unwrap_or(3);

            create_batch_vms(&hyperv, prefix, template_path, count)?;
        }

        // =================================================================
        // List available templates
        // =================================================================
        "list-templates" => {
            let template_dir = args
                .get(2)
                .map(|s| s.as_str())
                .unwrap_or("C:\\Hyper-V\\Templates");

            list_templates(template_dir)?;
        }

        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn create_windows_vm(
    hyperv: &HyperV,
    vm_name: &str,
    template_path: &str,
    memory_mb: u64,
    cpu_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Windows VM '{}'...\n", vm_name);

    // Determine VM directory
    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

    // Create directory
    std::fs::create_dir_all(&vm_dir)?;

    // Copy template VHDX
    println!("  Copying template VHDX...");
    println!("    From: {}", template_path);
    println!("    To:   {}", vhdx_path);
    std::fs::copy(template_path, &vhdx_path)?;

    // Create VM
    println!("  Creating Gen2 VM...");
    let vm = hyperv.create_vm(
        vm_name,
        memory_mb,
        cpu_count,
        VmGeneration::Gen2,
        Some(&vhdx_path),
    )?;

    println!();
    println!("Windows VM '{}' created successfully!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Memory: {} MB", memory_mb);
    println!("  CPUs:   {}", cpu_count);
    println!("  VHDX:   {}", vhdx_path);
    println!();
    println!(
        "Start with: cargo run --example vm_lifecycle -- start {}",
        vm_name
    );

    Ok(())
}

#[cfg(windows)]
fn create_linux_vm(
    hyperv: &HyperV,
    vm_name: &str,
    template_path: &str,
    memory_mb: u64,
    cpu_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Linux VM '{}'...\n", vm_name);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Copy template
    println!("  Copying template VHDX...");
    std::fs::copy(template_path, &vhdx_path)?;

    // Create Gen2 VM (Linux works well with Gen2 + Secure Boot disabled)
    println!("  Creating Gen2 VM...");
    let vm = hyperv.create_vm(
        vm_name,
        memory_mb,
        cpu_count,
        VmGeneration::Gen2,
        Some(&vhdx_path),
    )?;

    // Disable Secure Boot for Linux (via PowerShell)
    println!("  Disabling Secure Boot for Linux compatibility...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Set-VMFirmware -VMName '{}' -EnableSecureBoot Off", vm_name),
        ])
        .output()?;

    println!();
    println!("Linux VM '{}' created successfully!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Memory: {} MB", memory_mb);
    println!("  CPUs:   {}", cpu_count);
    println!("  VHDX:   {}", vhdx_path);
    println!("  Secure Boot: Disabled");
    println!();
    println!(
        "Start with: cargo run --example vm_lifecycle -- start {}",
        vm_name
    );

    Ok(())
}

#[cfg(windows)]
fn create_windows_clone(
    hyperv: &HyperV,
    vm_name: &str,
    parent_vhdx: &str,
    memory_mb: u64,
    cpu_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Windows VM '{}' (differencing disk)...\n", vm_name);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Create differencing disk (very fast, only stores changes)
    println!("  Creating differencing disk...");
    println!("    Parent: {}", parent_vhdx);
    println!("    Child:  {}", vhdx_path);
    hyperv.create_differencing_vhd(&vhdx_path, parent_vhdx)?;

    // Create VM
    println!("  Creating Gen2 VM...");
    let vm = hyperv.create_vm(
        vm_name,
        memory_mb,
        cpu_count,
        VmGeneration::Gen2,
        Some(&vhdx_path),
    )?;

    println!();
    println!("Windows VM '{}' created with differencing disk!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Parent: {}", parent_vhdx);
    println!("  Note:   Changes are stored separately, parent is read-only");
    println!();
    println!(
        "Start with: cargo run --example vm_lifecycle -- start {}",
        vm_name
    );

    Ok(())
}

#[cfg(windows)]
fn create_linux_clone(
    hyperv: &HyperV,
    vm_name: &str,
    parent_vhdx: &str,
    memory_mb: u64,
    cpu_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Linux VM '{}' (differencing disk)...\n", vm_name);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Create differencing disk
    println!("  Creating differencing disk...");
    hyperv.create_differencing_vhd(&vhdx_path, parent_vhdx)?;

    // Create VM
    println!("  Creating Gen2 VM...");
    let vm = hyperv.create_vm(
        vm_name,
        memory_mb,
        cpu_count,
        VmGeneration::Gen2,
        Some(&vhdx_path),
    )?;

    // Disable Secure Boot
    println!("  Disabling Secure Boot...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Set-VMFirmware -VMName '{}' -EnableSecureBoot Off", vm_name),
        ])
        .output()?;

    println!();
    println!("Linux VM '{}' created with differencing disk!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Parent: {}", parent_vhdx);
    println!();
    println!(
        "Start with: cargo run --example vm_lifecycle -- start {}",
        vm_name
    );

    Ok(())
}

#[cfg(windows)]
fn create_dev_windows_vm(
    hyperv: &HyperV,
    vm_name: &str,
    template_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Windows Development VM '{}'...\n", vm_name);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let os_vhdx = format!("{}\\{}-OS.vhdx", vm_dir, vm_name);
    let data_vhdx = format!("{}\\{}-Data.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Copy OS template
    println!("  Copying OS template...");
    std::fs::copy(template_path, &os_vhdx)?;

    // Create data disk (100GB for dev tools, repos, etc.)
    println!("  Creating 100GB data disk...");
    let data_size = 100 * 1024 * 1024 * 1024u64;
    hyperv.create_vhd(&data_vhdx, data_size, VhdType::Dynamic, None)?;

    // Create VM with generous resources
    println!("  Creating VM (8GB RAM, 4 CPUs)...");
    let vm = hyperv.create_vm(vm_name, 8192, 4, VmGeneration::Gen2, Some(&os_vhdx))?;

    // Attach data disk
    println!("  Attaching data disk...");
    hyperv.add_hard_disk_drive(vm_name, &data_vhdx)?;

    // Enable nested virtualization for Docker/WSL2
    println!("  Enabling nested virtualization...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Set-VMProcessor -VMName '{}' -ExposeVirtualizationExtensions $true",
                vm_name
            ),
        ])
        .output()?;

    // Enable dynamic memory
    println!("  Configuring dynamic memory (4-16GB)...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Set-VMMemory -VMName '{}' -DynamicMemoryEnabled $true -MinimumBytes 4GB -MaximumBytes 16GB",
                vm_name
            ),
        ])
        .output()?;

    println!();
    println!("Development VM '{}' created!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Memory: 4-16 GB (dynamic)");
    println!("  CPUs:   4");
    println!("  OS:     {}", os_vhdx);
    println!("  Data:   {} (100GB)", data_vhdx);
    println!("  Features: Nested virtualization enabled");
    println!();
    println!("Note: Initialize the data disk after first boot:");
    println!("  - Open Disk Management in Windows");
    println!("  - Initialize and format the new disk");

    Ok(())
}

#[cfg(windows)]
fn create_dev_linux_vm(
    hyperv: &HyperV,
    vm_name: &str,
    template_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Linux Development VM '{}'...\n", vm_name);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let os_vhdx = format!("{}\\{}-OS.vhdx", vm_dir, vm_name);
    let data_vhdx = format!("{}\\{}-Data.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Copy OS template
    println!("  Copying OS template...");
    std::fs::copy(template_path, &os_vhdx)?;

    // Create data disk
    println!("  Creating 100GB data disk...");
    let data_size = 100 * 1024 * 1024 * 1024u64;
    hyperv.create_vhd(&data_vhdx, data_size, VhdType::Dynamic, None)?;

    // Create VM
    println!("  Creating VM (8GB RAM, 4 CPUs)...");
    let vm = hyperv.create_vm(vm_name, 8192, 4, VmGeneration::Gen2, Some(&os_vhdx))?;

    // Disable Secure Boot
    println!("  Disabling Secure Boot...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Set-VMFirmware -VMName '{}' -EnableSecureBoot Off", vm_name),
        ])
        .output()?;

    // Attach data disk
    println!("  Attaching data disk...");
    hyperv.add_hard_disk_drive(vm_name, &data_vhdx)?;

    // Enable nested virtualization
    println!("  Enabling nested virtualization...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Set-VMProcessor -VMName '{}' -ExposeVirtualizationExtensions $true",
                vm_name
            ),
        ])
        .output()?;

    // Dynamic memory
    println!("  Configuring dynamic memory (2-16GB)...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Set-VMMemory -VMName '{}' -DynamicMemoryEnabled $true -MinimumBytes 2GB -MaximumBytes 16GB",
                vm_name
            ),
        ])
        .output()?;

    println!();
    println!("Linux Development VM '{}' created!", vm_name);
    println!("  ID:     {}", vm.id());
    println!("  Memory: 2-16 GB (dynamic)");
    println!("  CPUs:   4");
    println!("  OS:     {}", os_vhdx);
    println!("  Data:   {} (100GB)", data_vhdx);
    println!();
    println!("After first boot, format the data disk:");
    println!("  sudo fdisk /dev/sdb");
    println!("  sudo mkfs.ext4 /dev/sdb1");
    println!("  sudo mount /dev/sdb1 /data");

    Ok(())
}

#[cfg(windows)]
fn create_server_vm(
    hyperv: &HyperV,
    vm_name: &str,
    template_path: &str,
    os_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Server VM '{}' ({})...\n", vm_name, os_type);

    let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
    let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

    std::fs::create_dir_all(&vm_dir)?;

    // Copy template
    println!("  Copying template...");
    std::fs::copy(template_path, &vhdx_path)?;

    // Create VM with server-grade resources
    println!("  Creating VM (16GB RAM, 8 CPUs)...");
    let vm = hyperv.create_vm(vm_name, 16384, 8, VmGeneration::Gen2, Some(&vhdx_path))?;

    if os_type == "linux" {
        println!("  Disabling Secure Boot...");
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Set-VMFirmware -VMName '{}' -EnableSecureBoot Off", vm_name),
            ])
            .output()?;
    }

    // Configure for server workload
    println!("  Configuring for server workload...");
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                Set-VM -Name '{}' -AutomaticStartAction Start -AutomaticStopAction ShutDown
                Set-VMMemory -VMName '{}' -DynamicMemoryEnabled $true -MinimumBytes 8GB -MaximumBytes 32GB
                "#,
                vm_name, vm_name
            ),
        ])
        .output()?;

    println!();
    println!("Server VM '{}' created!", vm_name);
    println!("  ID:       {}", vm.id());
    println!("  Memory:   8-32 GB (dynamic)");
    println!("  CPUs:     8");
    println!("  Auto Start: Yes");
    println!("  Auto Stop:  Graceful shutdown");

    Ok(())
}

#[cfg(windows)]
fn create_batch_vms(
    hyperv: &HyperV,
    prefix: &str,
    template_path: &str,
    count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating {} VMs with prefix '{}'...\n", count, prefix);

    for i in 1..=count {
        let vm_name = format!("{}-{:02}", prefix, i);
        let vm_dir = format!("C:\\Hyper-V\\Virtual Machines\\{}", vm_name);
        let vhdx_path = format!("{}\\{}.vhdx", vm_dir, vm_name);

        println!("Creating VM {}/{}...", i, count);

        std::fs::create_dir_all(&vm_dir)?;

        // Use differencing disk for fast creation
        hyperv.create_differencing_vhd(&vhdx_path, template_path)?;

        hyperv.create_vm(&vm_name, 2048, 2, VmGeneration::Gen2, Some(&vhdx_path))?;

        println!("  Created: {}", vm_name);
    }

    println!();
    println!(
        "Created {} VMs: {}-01 to {}-{:02}",
        count, prefix, prefix, count
    );
    println!();
    println!("Note: All VMs use differencing disks based on:");
    println!("  {}", template_path);

    Ok(())
}

#[cfg(windows)]
fn list_templates(template_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for VHDX templates in: {}\n", template_dir);

    let path = Path::new(template_dir);
    if !path.exists() {
        println!("Directory does not exist.");
        println!();
        println!("Create it and add your template VHDX files:");
        println!("  mkdir \"{}\"", template_dir);
        return Ok(());
    }

    let mut found = false;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "vhdx").unwrap_or(false) {
            found = true;
            let name = path.file_name().unwrap().to_string_lossy();
            let metadata = std::fs::metadata(&path)?;
            let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
            println!("  {:<40} {:>8.1} GB", name, size_gb);
        }
    }

    if !found {
        println!("  No .vhdx files found.");
        println!();
        println!("Add template VHDX files to this directory.");
        println!("You can create templates using:");
        println!("  cargo run --example create_vm -- from-iso ...");
    }

    Ok(())
}

#[cfg(windows)]
fn print_usage(program: &str) {
    eprintln!("Usage: {} <command> [args...]", program);
    eprintln!();
    eprintln!("Create VMs from OS Templates:");
    eprintln!("  windows <name> <template.vhdx> [mem_mb] [cpus]");
    eprintln!("                              - Create Windows VM (copy template)");
    eprintln!("  linux <name> <template.vhdx> [mem_mb] [cpus]");
    eprintln!("                              - Create Linux VM (copy template)");
    eprintln!();
    eprintln!("Create VMs with Differencing Disks (fast clone):");
    eprintln!("  windows-clone <name> <parent.vhdx> [mem_mb] [cpus]");
    eprintln!("                              - Windows VM using differencing disk");
    eprintln!("  linux-clone <name> <parent.vhdx> [mem_mb] [cpus]");
    eprintln!("                              - Linux VM using differencing disk");
    eprintln!();
    eprintln!("Create Development VMs (more resources):");
    eprintln!("  dev-windows [name] <template.vhdx>");
    eprintln!("                              - Windows dev VM (8GB, 4 CPUs, data disk)");
    eprintln!("  dev-linux [name] <template.vhdx>");
    eprintln!("                              - Linux dev VM (8GB, 4 CPUs, data disk)");
    eprintln!();
    eprintln!("Create Server VMs:");
    eprintln!("  server <name> <template.vhdx> [windows|linux]");
    eprintln!("                              - Server VM (16GB, 8 CPUs, auto-start)");
    eprintln!();
    eprintln!("Batch Operations:");
    eprintln!("  batch <prefix> <template.vhdx> [count]");
    eprintln!("                              - Create multiple VMs (differencing disks)");
    eprintln!("  list-templates [directory]  - List available template VHDXs");
    eprintln!();
    eprintln!("Examples:");
    eprintln!();
    eprintln!("  # Create Windows VM from template");
    eprintln!(
        "  {} windows Win11-Test \"C:\\Templates\\Win11-Base.vhdx\" 4096 2",
        program
    );
    eprintln!();
    eprintln!("  # Create Linux VM (Ubuntu)");
    eprintln!(
        "  {} linux Ubuntu-Dev \"C:\\Templates\\Ubuntu-22.04.vhdx\" 2048 2",
        program
    );
    eprintln!();
    eprintln!("  # Quick clone for testing (uses parent disk, very fast)");
    eprintln!(
        "  {} windows-clone TestVM \"C:\\Templates\\Win11-Base.vhdx\"",
        program
    );
    eprintln!();
    eprintln!("  # Create development environment");
    eprintln!(
        "  {} dev-windows DevBox \"C:\\Templates\\Win11-Base.vhdx\"",
        program
    );
    eprintln!();
    eprintln!("  # Create 5 test VMs");
    eprintln!(
        "  {} batch TestEnv \"C:\\Templates\\Win11-Base.vhdx\" 5",
        program
    );
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
