# Install Hyper-V
Install-WindowsFeature -Name Hyper-V -IncludeManagementTools -Restart:$false
        
# Install Failover Clustering
Install-WindowsFeature -Name Failover-Clustering -IncludeManagementTools
        
# Install additional tools
Install-WindowsFeature -Name RSAT-Clustering-PowerShell
Install-WindowsFeature -Name Hyper-V-PowerShell

# install rust
winget install Rustlang.Rustup
rustup toolchain install 1.82.0
rustup default 1.82.0

Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile vs_buildtools.exe

# Silent install with C++ tools
./vs_buildtools.exe --passive --wait --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.Windows11SDK.22621


After installation completes, MSVC should be at:

C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\

winget install Microsoft.PowerShell


        
# Create marker file
New-Item -Path C:\features_installed.txt -ItemType File

# Enable WSL feature
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart

# Enable Virtual Machine Platform (required for WSL 2)
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart


# Check common reboot-pending registry keys
$rebootRequired = Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending"
$rebootRequired = $rebootRequired -or (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired")
$rebootRequired

# Reboot
Restart-Computer

wsl --list --online

# After reboot, set WSL 2 as default
wsl --set-default-version 2

# Install a distro
wsl --install -d Ubuntu-24.04


# run this after reboot
New-Cluster -Name "s-Cluster" -Node $env:COMPUTERNAME -StaticAddress 172.16.0.100 -NoStorage -Force

# Verify
Get-Cluster
Get-ClusterNode
Get-Service ClusSvc  # Should be Running now

# Test if .100 is available
Test-Connection 172.16.0.100 -Count 1 -Quiet  # Should return False (unreachable = available)


# setup cluster permission
Add-ClusterResourceDependency -Resource "Cluster Name" -Provider "DOMAIN\ServiceAccount"