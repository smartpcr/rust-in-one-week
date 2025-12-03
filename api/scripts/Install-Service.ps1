#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Installs the Windows Infrastructure API as a Windows Service.

.DESCRIPTION
    This script installs, uninstalls, or manages the WinInfraApi Windows service.
    It reads configuration from config.toml to determine service name, install path,
    and other settings.

.PARAMETER Action
    The action to perform: Install, Uninstall, Start, Stop, Restart, or Status.
    Default: Install

.PARAMETER ConfigPath
    Path to the config.toml file. If not specified, looks in the script directory
    or uses defaults.

.PARAMETER ExePath
    Path to the api.exe executable.
    Default: ..\target\release\api.exe (relative to script location)

.EXAMPLE
    .\Install-Service.ps1 -Action Install
    Installs the service with settings from config.toml.

.EXAMPLE
    .\Install-Service.ps1 -Action Install -ConfigPath "C:\config\myapi.toml"
    Installs the service using a custom config file.

.EXAMPLE
    .\Install-Service.ps1 -Action Uninstall
    Uninstalls the service and optionally removes files.

.EXAMPLE
    .\Install-Service.ps1 -Action Status
    Shows the current service status.
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet("Install", "Uninstall", "Start", "Stop", "Restart", "Status")]
    [string]$Action = "Install",

    [Parameter()]
    [string]$ConfigPath,

    [Parameter()]
    [string]$ExePath
)

$ErrorActionPreference = "Stop"

# Resolve script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Resolve executable path
if (-not $ExePath) {
    $ExePath = Join-Path $scriptDir "..\target\release\api.exe"
}
$ExePath = [System.IO.Path]::GetFullPath($ExePath)

# Default configuration values
$defaultConfig = @{
    ServiceName = "nodeagent"
    DisplayName = "Node Agent"
    Description = "REST API for Windows Failover Cluster and Hyper-V management"
    InstallPath = "C:\Program Files\azurestack\nodeagent"
    Host = "0.0.0.0"
    Port = 6001
    LogLevel = "api=info,tower_http=info"
}

function Write-Status {
    param([string]$Message, [string]$Type = "Info")

    switch ($Type) {
        "Info"    { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
        "Success" { Write-Host "[OK] $Message" -ForegroundColor Green }
        "Warning" { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
        "Error"   { Write-Host "[ERROR] $Message" -ForegroundColor Red }
    }
}

function Parse-TomlFile {
    param([string]$Path)

    $config = @{}

    if (-not (Test-Path $Path)) {
        return $config
    }

    $content = Get-Content $Path -Raw
    $currentSection = ""

    foreach ($line in $content -split "`n") {
        $line = $line.Trim()

        # Skip empty lines and comments
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith("#")) {
            continue
        }

        # Section header
        if ($line -match '^\[(.+)\]$') {
            $currentSection = $Matches[1]
            continue
        }

        # Key-value pair
        if ($line -match '^(\w+)\s*=\s*(.+)$') {
            $key = $Matches[1]
            $value = $Matches[2].Trim()

            # Remove quotes from string values
            if ($value -match '^"(.+)"$' -or $value -match "^'(.+)'$") {
                $value = $Matches[1]
            }

            # Convert to appropriate type
            if ($value -match '^\d+$') {
                $value = [int]$value
            }

            $fullKey = if ($currentSection) { "${currentSection}.${key}" } else { $key }
            $config[$fullKey] = $value
        }
    }

    return $config
}

function Get-Configuration {
    # Find config file
    $configFile = $ConfigPath
    if (-not $configFile) {
        # Look in script directory first
        $configFile = Join-Path $scriptDir "config.toml"
        if (-not (Test-Path $configFile)) {
            # Look in parent directory (api folder)
            $configFile = Join-Path $scriptDir "..\config.toml"
        }
    }

    $config = $defaultConfig.Clone()

    if (Test-Path $configFile) {
        Write-Status "Loading configuration from: $configFile"
        $toml = Parse-TomlFile $configFile

        # Map TOML values to config
        if ($toml["service.name"]) { $config.ServiceName = $toml["service.name"] }
        if ($toml["service.display_name"]) { $config.DisplayName = $toml["service.display_name"] }
        if ($toml["service.description"]) { $config.Description = $toml["service.description"] }
        if ($toml["service.install_path"]) { $config.InstallPath = $toml["service.install_path"] }
        if ($toml["server.host"]) { $config.Host = $toml["server.host"] }
        if ($toml["server.port"]) { $config.Port = $toml["server.port"] }
        if ($toml["logging.level"]) { $config.LogLevel = $toml["logging.level"] }
    } else {
        Write-Status "No config file found, using defaults" -Type Warning
    }

    return $config
}

function Test-ServiceExists {
    param([string]$Name)
    $service = Get-Service -Name $Name -ErrorAction SilentlyContinue
    return $null -ne $service
}

function Get-ServiceStatus {
    param([string]$Name)
    if (Test-ServiceExists $Name) {
        $service = Get-Service -Name $Name
        return $service.Status
    }
    return "NotInstalled"
}

function Install-ApiService {
    $config = Get-Configuration

    Write-Status "Installing $($config.DisplayName)..."
    Write-Status "  Service Name: $($config.ServiceName)"
    Write-Status "  Install Path: $($config.InstallPath)"

    # Check if executable exists
    if (-not (Test-Path $ExePath)) {
        Write-Status "Executable not found at: $ExePath" -Type Error
        Write-Status "Please build the project first with: cargo build --release -p api" -Type Info
        exit 1
    }

    # Check if service already exists
    if (Test-ServiceExists $config.ServiceName) {
        Write-Status "Service '$($config.ServiceName)' already exists. Use -Action Uninstall first." -Type Warning
        exit 1
    }

    # Create installation directory
    if (-not (Test-Path $config.InstallPath)) {
        Write-Status "Creating installation directory: $($config.InstallPath)"
        New-Item -ItemType Directory -Path $config.InstallPath -Force | Out-Null
    }

    # Copy executable
    $destExe = Join-Path $config.InstallPath "api.exe"
    Write-Status "Copying executable to: $destExe"
    Copy-Item -Path $ExePath -Destination $destExe -Force

    # Create config file in install directory
    $destConfig = Join-Path $config.InstallPath "config.toml"
    Write-Status "Creating configuration: $destConfig"

    $configContent = @"
# Windows Infrastructure API Configuration
# Generated by Install-Service.ps1

[server]
# Host address to bind to
host = "$($config.Host)"
# Port to listen on
port = $($config.Port)

[logging]
# Log level filter
level = "$($config.LogLevel)"

[service]
# Windows service name (must match installed service)
name = "$($config.ServiceName)"
# Display name shown in Services console
display_name = "$($config.DisplayName)"
# Service description
description = "$($config.Description)"
# Installation directory
install_path = "$($config.InstallPath -replace '\\', '\\')"
"@
    Set-Content -Path $destConfig -Value $configContent -Encoding UTF8

    # Create the Windows service
    Write-Status "Creating Windows service: $($config.ServiceName)"
    $binPath = "`"$destExe`" --service"

    $service = New-Service -Name $config.ServiceName `
        -BinaryPathName $binPath `
        -DisplayName $config.DisplayName `
        -Description $config.Description `
        -StartupType Automatic

    Write-Status "Service installed successfully!" -Type Success
    Write-Status ""
    Write-Status "Installation Summary:" -Type Info
    Write-Status "  Service Name:  $($config.ServiceName)"
    Write-Status "  Display Name:  $($config.DisplayName)"
    Write-Status "  Install Path:  $($config.InstallPath)"
    Write-Status "  Config File:   $destConfig"
    Write-Status "  Listen:        $($config.Host):$($config.Port)"
    Write-Status ""
    Write-Status "To start the service, run: .\Install-Service.ps1 -Action Start" -Type Info
}

function Uninstall-ApiService {
    $config = Get-Configuration

    Write-Status "Uninstalling $($config.DisplayName)..."

    if (-not (Test-ServiceExists $config.ServiceName)) {
        Write-Status "Service '$($config.ServiceName)' is not installed." -Type Warning
        return
    }

    # Stop the service if running
    $status = Get-ServiceStatus $config.ServiceName
    if ($status -eq "Running") {
        Write-Status "Stopping service..."
        Stop-Service -Name $config.ServiceName -Force
        Start-Sleep -Seconds 2
    }

    # Remove the service
    Write-Status "Removing service registration..."
    sc.exe delete $config.ServiceName | Out-Null

    # Ask about removing files
    if (Test-Path $config.InstallPath) {
        $response = Read-Host "Remove installation directory '$($config.InstallPath)'? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Write-Status "Removing installation directory..."
            Remove-Item -Path $config.InstallPath -Recurse -Force
        }
    }

    Write-Status "Service uninstalled successfully!" -Type Success
}

function Start-ApiService {
    $config = Get-Configuration

    if (-not (Test-ServiceExists $config.ServiceName)) {
        Write-Status "Service '$($config.ServiceName)' is not installed." -Type Error
        exit 1
    }

    $status = Get-ServiceStatus $config.ServiceName
    if ($status -eq "Running") {
        Write-Status "Service is already running." -Type Warning
        return
    }

    Write-Status "Starting service '$($config.ServiceName)'..."
    Start-Service -Name $config.ServiceName
    Start-Sleep -Seconds 2

    $status = Get-ServiceStatus $config.ServiceName
    if ($status -eq "Running") {
        Write-Status "Service started successfully!" -Type Success
    } else {
        Write-Status "Service failed to start. Check Event Viewer for details." -Type Error
    }
}

function Stop-ApiService {
    $config = Get-Configuration

    if (-not (Test-ServiceExists $config.ServiceName)) {
        Write-Status "Service '$($config.ServiceName)' is not installed." -Type Error
        exit 1
    }

    $status = Get-ServiceStatus $config.ServiceName
    if ($status -ne "Running") {
        Write-Status "Service is not running." -Type Warning
        return
    }

    Write-Status "Stopping service '$($config.ServiceName)'..."
    Stop-Service -Name $config.ServiceName -Force
    Start-Sleep -Seconds 2

    Write-Status "Service stopped." -Type Success
}

function Restart-ApiService {
    Stop-ApiService
    Start-ApiService
}

function Show-ServiceStatus {
    $config = Get-Configuration

    Write-Host ""
    Write-Host "Service Status" -ForegroundColor White
    Write-Host "==============" -ForegroundColor White

    if (-not (Test-ServiceExists $config.ServiceName)) {
        Write-Host "Service:  Not Installed" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Expected Configuration:" -ForegroundColor Gray
        Write-Host "  Name:         $($config.ServiceName)"
        Write-Host "  Display:      $($config.DisplayName)"
        Write-Host "  Install Path: $($config.InstallPath)"
        return
    }

    $service = Get-Service -Name $config.ServiceName
    $statusColor = switch ($service.Status) {
        "Running" { "Green" }
        "Stopped" { "Red" }
        default { "Yellow" }
    }

    Write-Host "Name:     $($config.ServiceName)"
    Write-Host "Display:  $($service.DisplayName)"
    Write-Host "Status:   $($service.Status)" -ForegroundColor $statusColor
    Write-Host "Startup:  $($service.StartType)"

    $destExe = Join-Path $config.InstallPath "api.exe"
    if (Test-Path $destExe) {
        Write-Host "Path:     $destExe"
    }

    $destConfig = Join-Path $config.InstallPath "config.toml"
    if (Test-Path $destConfig) {
        Write-Host "Config:   $destConfig"
    }

    Write-Host ""
}

# Execute the requested action
switch ($Action) {
    "Install"   { Install-ApiService }
    "Uninstall" { Uninstall-ApiService }
    "Start"     { Start-ApiService }
    "Stop"      { Stop-ApiService }
    "Restart"   { Restart-ApiService }
    "Status"    { Show-ServiceStatus }
}
