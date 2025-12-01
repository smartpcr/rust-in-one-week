# Self-Hosted Runner Setup for Cluster Tests

This document explains how to set up a self-hosted GitHub Actions runner for running the Windows Failover Cluster integration tests.

## Prerequisites

### Hardware/VM Requirements
- Windows Server 2019 or 2022
- At least 4 GB RAM
- At least 2 CPU cores
- Network connectivity to GitHub

### Software Requirements
1. **Failover Clustering Feature**
   ```powershell
   # Install Failover Clustering with management tools
   Install-WindowsFeature -Name Failover-Clustering -IncludeManagementTools

   # Restart if required
   Restart-Computer
   ```

2. **Cluster Membership**
   The server must be a member of a Windows Failover Cluster. For testing purposes, you can create a single-node cluster:
   ```powershell
   # Create a single-node cluster (for testing only)
   New-Cluster -Name "TestCluster" -Node $env:COMPUTERNAME -NoStorage -AdministrativeAccessPoint Dns
   ```

3. **Rust Toolchain**
   ```powershell
   # Download and run rustup-init.exe from https://rustup.rs
   # Or use winget:
   winget install Rustlang.Rustup
   ```

## Setting Up the GitHub Actions Runner

### Step 1: Create Runner in GitHub
1. Go to your repository on GitHub
2. Navigate to **Settings** > **Actions** > **Runners**
3. Click **New self-hosted runner**
4. Select **Windows** as the operating system
5. Follow the download and configuration instructions

### Step 2: Configure the Runner
```powershell
# Navigate to the runner directory
cd C:\actions-runner

# Configure the runner with required labels
.\config.cmd --url https://github.com/YOUR_ORG/YOUR_REPO --token YOUR_TOKEN --labels self-hosted,windows,cluster
```

### Step 3: Install as Windows Service
```powershell
# Install the runner as a Windows service
.\svc.cmd install

# Configure the service to run with an account that has cluster admin privileges
# Option 1: Use a domain account with cluster permissions
.\svc.cmd install DOMAIN\ServiceAccount

# Option 2: Use Local System (if the machine is a cluster node with admin rights)
.\svc.cmd install

# Start the service
.\svc.cmd start
```

### Step 4: Verify Cluster Permissions
The service account running the GitHub Actions runner must have:
- Cluster Administrator permissions
- Local Administrator on the cluster nodes

```powershell
# Grant cluster permissions to the service account
Add-ClusterResourceDependency -Resource "Cluster Name" -Provider "DOMAIN\ServiceAccount"

# Or add to cluster administrators
Grant-ClusterAccess -User "DOMAIN\ServiceAccount" -Full
```

## Testing the Setup

### Manual Verification
```powershell
# Verify Failover Clustering is installed
Get-WindowsFeature -Name Failover-Clustering

# Verify cluster service is running
Get-Service -Name ClusSvc

# Verify cluster connectivity
Get-Cluster
Get-ClusterNode
Get-ClusterGroup
```

### Run Tests Locally
```powershell
# Clone the repository
git clone https://github.com/YOUR_ORG/YOUR_REPO.git
cd YOUR_REPO

# Build clus module
cargo build -p clus

# Run unit tests
cargo test -p clus

# Run integration tests (requires cluster)
cargo test -p clus -- --ignored
```

## Troubleshooting

### Cluster Service Not Running
```powershell
# Check service status
Get-Service -Name ClusSvc

# Start the service
Start-Service -Name ClusSvc

# Check for errors in event log
Get-EventLog -LogName System -Source "Microsoft-Windows-FailoverClustering" -Newest 10
```

### Permission Denied Errors
```powershell
# Check current user's cluster permissions
Get-ClusterAccess

# Grant full access
Grant-ClusterAccess -User $env:USERNAME -Full
```

### Runner Not Picking Up Jobs
1. Check runner status in GitHub (Settings > Actions > Runners)
2. Verify the runner service is running:
   ```powershell
   Get-Service -Name "actions.runner.*"
   ```
3. Check runner logs:
   ```powershell
   Get-Content C:\actions-runner\_diag\Runner_*.log -Tail 50
   ```

## Security Considerations

1. **Network Isolation**: Consider running the self-hosted runner in an isolated network segment
2. **Least Privilege**: Only grant the minimum required cluster permissions
3. **Trusted Code Only**: The workflow is configured to only run cluster tests for trusted code (not from forks)
4. **Regular Updates**: Keep Windows Server and the runner updated with security patches

## Workflow Labels

The workflows expect these labels on the self-hosted runner:
- `self-hosted` - Required for all self-hosted runners
- `windows` - Identifies Windows runners
- `cluster` - Identifies runners with Failover Clustering
