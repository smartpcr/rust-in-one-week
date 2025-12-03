# Create a folder under the drive root
mkdir c:\actions-runner; 
cd c:\actions-runner

# Download the latest runner package
Invoke-WebRequest -Uri https://github.com/actions/runner/releases/download/v2.329.0/actions-runner-win-x64-2.329.0.zip -OutFile actions-runner-win-x64-2.329.0.zip

# Optional: Validate the hash
if((Get-FileHash -Path actions-runner-win-x64-2.329.0.zip -Algorithm SHA256).Hash.ToUpper() -ne 'f60be5ddf373c52fd735388c3478536afd12bfd36d1d0777c6b855b758e70f25'.ToUpper()){ 
  throw 'Computed checksum did not match' 
}

# Extract the installer
Add-Type -AssemblyName System.IO.Compression.FileSystem ; 
[System.IO.Compression.ZipFile]::ExtractToDirectory("$PWD/actions-runner-win-x64-2.329.0.zip", "$PWD")


# Create the runner and start the configuration experience, make sure to run service as current user, 
# not LocalSystem or network service, also enter a custom label 
$gh_token = "enter here"
./config.cmd --url https://github.com/smartpcr/rust-in-one-week --token $gh_token


# Check the runner config file
Get-Content "C:\actions-runner\.runner" | ConvertFrom-Json

./config.cmd remove --token $gh_token

./config.cmd --url https://github.com/smartpcr/rust-in-one-week --token $gh_token



# setup permissions
# Run PowerShell as Administrator

# Add azureuser to Hyper-V Administrators
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "azureuser"

# Add LocalSystem (NT AUTHORITY\SYSTEM) to Hyper-V Administrators
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "NT AUTHORITY\SYSTEM"

# Add azureuser to Failover Cluster Administrators (if exists as local group)
# Note: Cluster permissions are typically managed differently

# For Failover Cluster, grant permissions via cluster command:
# Grant full cluster access to azureuser
Grant-ClusterAccess -User "azureuser" -Full

# Grant full cluster access to LocalSystem
Grant-ClusterAccess -User "NT AUTHORITY\SYSTEM" -Full

Verify the changes:
# Check Hyper-V Administrators group members
Get-LocalGroupMember -Group "Hyper-V Administrators"

# Check cluster access permissions
Get-ClusterAccess

#For a self-hosted GitHub runner running as LocalSystem:

#If you're setting this up in your CI/CD pipeline or runner setup script, you can add these to your runner provisioning:

# Setup script for Windows Server runner
# Run once during runner setup

# Hyper-V permissions
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "NT AUTHORITY\NETWORK SERVICE" -ErrorAction SilentlyContinue
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "azureuser" -ErrorAction SilentlyContinue

# Failover Cluster permissions
Grant-ClusterAccess -User "NT AUTHORITY\NETWORK SERVICE" -Full -ErrorAction SilentlyContinue
Grant-ClusterAccess -User "azureuser" -Full -ErrorAction SilentlyContinue

# Verify
Write-Host "Hyper-V Administrators:"
Get-LocalGroupMember -Group "Hyper-V Administrators"

Write-Host "Cluster Access:"
Get-ClusterAccess

# Verify GitHub Actions Runner Status
gh auth login
gh api repos/smartpcr/rust-in-one-week/actions/runners | jq ".runners[] | {name, status}"