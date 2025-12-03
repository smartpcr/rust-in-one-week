#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Installs the Windows Infrastructure API as a Windows Service.

.DESCRIPTION
    This script installs, uninstalls, or manages the WinInfraApi Windows service.
    It copies the executable and configuration to the specified installation directory
    and registers it as a Windows service.

.PARAMETER Action
    The action to perform: Install, Uninstall, Start, Stop, Restart, or Status.
    Default: Install

.PARAMETER InstallDir
    The directory where the service will be installed.
    Default: C:\Program Files\WinInfraApi

.PARAMETER ExePath
    Path to the api.exe executable.
    Default: ..\target\release\api.exe (relative to script location)

.PARAMETER ServiceName
    The name of the Windows service.
    Default: WinInfraApi

.PARAMETER DisplayName
    The display name of the Windows service.
    Default: Windows Infrastructure API

.PARAMETER Description
    The description of the Windows service.
    Default: REST API for Windows Failover Cluster and Hyper-V management

.EXAMPLE
    .\Install-Service.ps1 -Action Install
    Installs the service with default settings.

.EXAMPLE
    .\Install-Service.ps1 -Action Install -InstallDir "D:\Services\WinInfraApi"
    Installs the service to a custom directory.

.EXAMPLE
    .\Install-Service.ps1 -Action Uninstall
    Uninstalls the service and removes files.

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
    [string]$InstallDir = "C:\Program Files\WinInfraApi",

    [Parameter()]
    [string]$ExePath,

    [Parameter()]
    [string]$ServiceName = "WinInfraApi",

    [Parameter()]
    [string]$DisplayName = "Windows Infrastructure API",

    [Parameter()]
    [string]$Description = "REST API for Windows Failover Cluster and Hyper-V management"
)

$ErrorActionPreference = "Stop"

# Resolve executable path
if (-not $ExePath) {
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $ExePath = Join-Path $scriptDir "..\target\release\api.exe"
}

$ExePath = [System.IO.Path]::GetFullPath($ExePath)

function Write-Status {
    param([string]$Message, [string]$Type = "Info")

    switch ($Type) {
        "Info"    { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
        "Success" { Write-Host "[OK] $Message" -ForegroundColor Green }
        "Warning" { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
        "Error"   { Write-Host "[ERROR] $Message" -ForegroundColor Red }
    }
}

function Test-ServiceExists {
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    return $null -ne $service
}

function Get-ServiceStatus {
    if (Test-ServiceExists) {
        $service = Get-Service -Name $ServiceName
        return $service.Status
    }
    return "NotInstalled"
}

function Install-ApiService {
    Write-Status "Installing $DisplayName..."

    # Check if executable exists
    if (-not (Test-Path $ExePath)) {
        Write-Status "Executable not found at: $ExePath" -Type Error
        Write-Status "Please build the project first with: cargo build --release -p api" -Type Info
        exit 1
    }

    # Check if service already exists
    if (Test-ServiceExists) {
        Write-Status "Service '$ServiceName' already exists. Use -Action Uninstall first." -Type Warning
        exit 1
    }

    # Create installation directory
    if (-not (Test-Path $InstallDir)) {
        Write-Status "Creating installation directory: $InstallDir"
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Copy executable
    $destExe = Join-Path $InstallDir "api.exe"
    Write-Status "Copying executable to: $destExe"
    Copy-Item -Path $ExePath -Destination $destExe -Force

    # Create default config if it doesn't exist
    $configPath = Join-Path $InstallDir "config.toml"
    if (-not (Test-Path $configPath)) {
        Write-Status "Creating default configuration: $configPath"
        $defaultConfig = @"
# Windows Infrastructure API Configuration

[server]
# Host address to bind to
host = "0.0.0.0"
# Port to listen on
port = 3000

[logging]
# Log level filter (e.g., "api=info,tower_http=info", "debug", "trace")
level = "api=info,tower_http=info"
"@
        Set-Content -Path $configPath -Value $defaultConfig -Encoding UTF8
    }

    # Create the Windows service
    Write-Status "Creating Windows service: $ServiceName"
    $binPath = "`"$destExe`" --service"

    $service = New-Service -Name $ServiceName `
        -BinaryPathName $binPath `
        -DisplayName $DisplayName `
        -Description $Description `
        -StartupType Automatic

    Write-Status "Service installed successfully!" -Type Success
    Write-Status "Installation directory: $InstallDir" -Type Info
    Write-Status "Configuration file: $configPath" -Type Info
    Write-Status ""
    Write-Status "To start the service, run: .\Install-Service.ps1 -Action Start" -Type Info
    Write-Status "Or use: sc.exe start $ServiceName" -Type Info
}

function Uninstall-ApiService {
    Write-Status "Uninstalling $DisplayName..."

    if (-not (Test-ServiceExists)) {
        Write-Status "Service '$ServiceName' is not installed." -Type Warning
        return
    }

    # Stop the service if running
    $status = Get-ServiceStatus
    if ($status -eq "Running") {
        Write-Status "Stopping service..."
        Stop-Service -Name $ServiceName -Force
        Start-Sleep -Seconds 2
    }

    # Remove the service
    Write-Status "Removing service registration..."
    sc.exe delete $ServiceName | Out-Null

    # Ask about removing files
    if (Test-Path $InstallDir) {
        $response = Read-Host "Remove installation directory '$InstallDir'? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Write-Status "Removing installation directory..."
            Remove-Item -Path $InstallDir -Recurse -Force
        }
    }

    Write-Status "Service uninstalled successfully!" -Type Success
}

function Start-ApiService {
    if (-not (Test-ServiceExists)) {
        Write-Status "Service '$ServiceName' is not installed." -Type Error
        exit 1
    }

    $status = Get-ServiceStatus
    if ($status -eq "Running") {
        Write-Status "Service is already running." -Type Warning
        return
    }

    Write-Status "Starting service..."
    Start-Service -Name $ServiceName
    Start-Sleep -Seconds 2

    $status = Get-ServiceStatus
    if ($status -eq "Running") {
        Write-Status "Service started successfully!" -Type Success
    } else {
        Write-Status "Service failed to start. Check Event Viewer for details." -Type Error
    }
}

function Stop-ApiService {
    if (-not (Test-ServiceExists)) {
        Write-Status "Service '$ServiceName' is not installed." -Type Error
        exit 1
    }

    $status = Get-ServiceStatus
    if ($status -ne "Running") {
        Write-Status "Service is not running." -Type Warning
        return
    }

    Write-Status "Stopping service..."
    Stop-Service -Name $ServiceName -Force
    Start-Sleep -Seconds 2

    Write-Status "Service stopped." -Type Success
}

function Restart-ApiService {
    Stop-ApiService
    Start-ApiService
}

function Show-ServiceStatus {
    Write-Host ""
    Write-Host "Service Status" -ForegroundColor White
    Write-Host "==============" -ForegroundColor White

    if (-not (Test-ServiceExists)) {
        Write-Host "Service:  Not Installed" -ForegroundColor Yellow
        return
    }

    $service = Get-Service -Name $ServiceName
    $statusColor = switch ($service.Status) {
        "Running" { "Green" }
        "Stopped" { "Red" }
        default { "Yellow" }
    }

    Write-Host "Name:     $ServiceName"
    Write-Host "Display:  $($service.DisplayName)"
    Write-Host "Status:   $($service.Status)" -ForegroundColor $statusColor
    Write-Host "Startup:  $($service.StartType)"

    $destExe = Join-Path $InstallDir "api.exe"
    if (Test-Path $destExe) {
        Write-Host "Path:     $destExe"
    }

    $configPath = Join-Path $InstallDir "config.toml"
    if (Test-Path $configPath) {
        Write-Host "Config:   $configPath"
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
