param()

Write-Host "=== Checking Failover Clustering Feature ==="
$feature = Get-WindowsFeature -Name Failover-Clustering
if ($feature.InstallState -ne 'Installed') {
  Write-Error "FATAL: Failover Clustering feature is not installed"
  Write-Host "Install with: Install-WindowsFeature -Name Failover-Clustering -IncludeManagementTools"
  exit 1
}
Write-Host "✓ Failover Clustering feature is installed"

Write-Host ""
Write-Host "=== Checking Cluster Service ==="
$service = Get-Service -Name ClusSvc -ErrorAction SilentlyContinue
if (-not $service) {
  Write-Error "FATAL: Cluster service (ClusSvc) not found"
  exit 1
}

if ($service.Status -ne 'Running') {
  Write-Warning "Cluster service is not running (Status: $($service.Status))"
  Write-Host "Attempting to start cluster service..."
  Start-Service -Name ClusSvc -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 5
  $service = Get-Service -Name ClusSvc
}

if ($service.Status -eq 'Running') {
  Write-Host "✓ Cluster service is running"
} else {
  Write-Error "FATAL: Could not start cluster service"
  exit 1
}

Write-Host ""
Write-Host "=== Cluster Information ==="
try {
  $cluster = Get-Cluster -ErrorAction Stop
  Write-Host "✓ Connected to cluster: $($cluster.Name)"

  Write-Host ""
  Write-Host "Cluster Nodes:"
  Get-ClusterNode | ForEach-Object {
    Write-Host "  - $($_.Name): $($_.State)"
  }

  Write-Host ""
  Write-Host "Cluster Groups:"
  Get-ClusterGroup | ForEach-Object {
    Write-Host "  - $($_.Name): $($_.State) on $($_.OwnerNode)"
  }
} catch {
  Write-Error ("FATAL: Could not connect to cluster: {0}" -f $_)
  exit 1
}
