//! Example: Cluster Shared Volume (CSV) operations
//!
//! Demonstrates CSV management including:
//! - Listing all CSVs in the cluster
//! - Checking if a path is on a CSV
//! - Getting CSV volume information
//! - Setting maintenance mode

#[cfg(windows)]
use clus::{Cluster, Csv};
#[cfg(windows)]
use std::env;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = args[1].as_str();

    match command {
        // =================================================================
        // List all CSVs
        // =================================================================
        "list" => {
            let cluster_name = args.get(2).map(|s| s.as_str());
            let cluster = Cluster::open(cluster_name)?;

            println!("Cluster: {}", cluster.name()?);
            println!();

            let csvs = cluster.csv_volumes()?;
            if csvs.is_empty() {
                println!("No Cluster Shared Volumes found.");
            } else {
                println!("Cluster Shared Volumes:");
                println!("{:<30} {:<15} {:<20}", "Name", "State", "Owner Node");
                println!("{}", "-".repeat(65));

                for csv in &csvs {
                    let (state, owner) = csv.state()?;
                    println!(
                        "{:<30} {:<15?} {:<20}",
                        csv.name(),
                        state,
                        owner.as_deref().unwrap_or("-")
                    );
                }
            }
        }

        // =================================================================
        // Get CSV info
        // =================================================================
        "info" => {
            let cluster_name = args.get(2).map(|s| s.as_str());
            let cluster = Cluster::open(cluster_name)?;

            println!("Cluster: {}", cluster.name()?);
            println!();

            let csv_info = cluster.csv_info()?;
            if csv_info.is_empty() {
                println!("No Cluster Shared Volumes found.");
            } else {
                for info in &csv_info {
                    println!("CSV: {}", info.friendly_name);
                    println!("  Mount Point:      {}", info.mount_point);
                    println!("  State:            {:?}", info.state);
                    println!("  Fault State:      {:?}", info.fault_state);
                    println!("  Backup State:     {:?}", info.backup_state);
                    println!(
                        "  Owner Node:       {}",
                        info.owner_node.as_deref().unwrap_or("-")
                    );
                    println!("  Redirected I/O:   {:?}", info.redirected_io_reason);
                    println!("  Maintenance Mode: {}", info.in_maintenance);
                    println!();
                }
            }
        }

        // =================================================================
        // Check if path is on CSV
        // =================================================================
        "check-path" => {
            let path = args.get(2).ok_or("Missing path argument")?;

            let is_csv = Csv::is_path_on_csv(path);
            if is_csv {
                println!("Path '{}' IS on a Cluster Shared Volume", path);

                // Try to get the volume path
                match Csv::get_volume_path(path) {
                    Ok(vol_path) => println!("  Volume Path: {}", vol_path),
                    Err(e) => println!("  Could not get volume path: {}", e),
                }
            } else {
                println!("Path '{}' is NOT on a Cluster Shared Volume", path);
            }
        }

        // =================================================================
        // Get volume path for a file
        // =================================================================
        "volume-path" => {
            let path = args.get(2).ok_or("Missing path argument")?;

            match Csv::get_volume_path(path) {
                Ok(vol_path) => {
                    println!("File: {}", path);
                    println!("Volume Path: {}", vol_path);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // =================================================================
        // Get volume GUID name
        // =================================================================
        "volume-name" => {
            let mount_point = args.get(2).ok_or("Missing mount point argument")?;

            match Csv::get_volume_name(mount_point) {
                Ok(vol_name) => {
                    println!("Mount Point: {}", mount_point);
                    println!("Volume Name: {}", vol_name);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // =================================================================
        // Set maintenance mode
        // =================================================================
        "maintenance" => {
            let csv_name = args.get(2).ok_or("Missing CSV name")?;
            let enable = args
                .get(3)
                .map(|s| s == "on" || s == "true" || s == "1")
                .unwrap_or(true);

            let cluster = Cluster::open(None)?;
            let resource = cluster.open_resource(csv_name)?;

            // Verify it's a CSV
            if !Csv::is_csv_resource(&resource)? {
                eprintln!("Error: '{}' is not a Cluster Shared Volume", csv_name);
                std::process::exit(1);
            }

            let action = if enable { "Enabling" } else { "Disabling" };
            println!("{} maintenance mode for CSV '{}'...", action, csv_name);

            Csv::set_maintenance_mode(&resource, enable)?;

            println!(
                "Maintenance mode {} for CSV '{}'",
                if enable { "enabled" } else { "disabled" },
                csv_name
            );
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
fn print_usage(program: &str) {
    eprintln!("Usage: {} <command> [args...]", program);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  list [cluster_name]           - List all CSVs in the cluster");
    eprintln!("  info [cluster_name]           - Get detailed CSV information");
    eprintln!("  check-path <path>             - Check if a path is on a CSV");
    eprintln!("  volume-path <file_path>       - Get CSV volume path for a file");
    eprintln!("  volume-name <mount_point>     - Get volume GUID for a CSV mount point");
    eprintln!("  maintenance <csv_name> [on|off] - Set CSV maintenance mode");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} list", program);
    eprintln!("  {} list MyCluster", program);
    eprintln!("  {} info", program);
    eprintln!(
        "  {} check-path \"C:\\ClusterStorage\\Volume1\\data.txt\"",
        program
    );
    eprintln!(
        "  {} volume-path \"C:\\ClusterStorage\\Volume1\\data.txt\"",
        program
    );
    eprintln!(
        "  {} volume-name \"C:\\ClusterStorage\\Volume1\\\"",
        program
    );
    eprintln!("  {} maintenance \"Cluster Disk 1\" on", program);
    eprintln!("  {} maintenance \"Cluster Disk 1\" off", program);
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Failover Clustering installed.");
}
