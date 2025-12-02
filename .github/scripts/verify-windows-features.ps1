# Verify Windows Server Features for CI
# This script checks for Hyper-V and Failover Clustering features

Write-Host "=== Checking Hyper-V ==="
$hyperv = Get-WindowsFeature -Name Hyper-V
if ($hyperv.InstallState -eq 'Installed') {
    Write-Host "[OK] Hyper-V is installed"
} else {
    Write-Warning "Hyper-V is not installed - hv integration tests may fail"
}

Write-Host ""
Write-Host "=== Checking Failover Clustering Feature ==="
$feature = Get-WindowsFeature -Name Failover-Clustering
if ($feature.InstallState -eq 'Installed') {
    Write-Host "[OK] Failover Clustering feature is installed"
} else {
    Write-Warning "Failover Clustering is not installed - clus integration tests may fail"
}

Write-Host ""
Write-Host "=== Checking Cluster Service ==="
$service = Get-Service -Name ClusSvc -ErrorAction SilentlyContinue
if ($service -and $service.Status -eq 'Running') {
    Write-Host "[OK] Cluster service is running"
} else {
    Write-Warning "Cluster service is not running - clus integration tests may fail"
}

Write-Host ""
Write-Host "=== Cluster Information ==="
try {
    $cluster = Get-Cluster -ErrorAction Stop
    Write-Host "[OK] Connected to cluster: $($cluster.Name)"

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
}
catch {
    Write-Warning "Could not connect to cluster - clus integration tests may fail"
}
