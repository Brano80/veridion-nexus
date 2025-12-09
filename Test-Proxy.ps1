# Test-Proxy.ps1
# PowerShell script to test proxy endpoint with proper region display
# Usage: Test-Proxy "https://www.google.com"

function Test-Proxy {
    param(
        [Parameter(Mandatory=$true)]
        [string]$TargetUrl,
        
        [Parameter(Mandatory=$false)]
        [string]$AgentId = "test-agent-$(Get-Date -Format 'yyyyMMdd-HHmmss')",
        
        [Parameter(Mandatory=$false)]
        [string]$Method = "GET",
        
        [Parameter(Mandatory=$false)]
        [string]$ApiBase = "http://localhost:8080/api/v1"
    )
    
    # Get authentication token
    try {
        $loginResponse = Invoke-RestMethod -Uri "$ApiBase/auth/login" `
            -Method POST `
            -ContentType "application/json" `
            -Body (@{
                username = "admin"
                password = "admin123"
            } | ConvertTo-Json) `
            -ErrorAction Stop
        
        $token = $loginResponse.token
    } catch {
        Write-Host "❌ Authentication failed: $($_.Exception.Message)" -ForegroundColor Red
        return
    }
    
    # Prepare headers
    $headers = @{
        "Authorization" = "Bearer $token"
        "Content-Type" = "application/json"
        "X-Agent-ID" = $AgentId
    }
    
    # Prepare proxy request
    $proxyBody = @{
        target_url = $TargetUrl
        method = $Method
    } | ConvertTo-Json
    
    Write-Host "Connecting to: $TargetUrl ... " -NoNewline
    
    # Test proxy endpoint
    try {
        $response = Invoke-RestMethod -Uri "$ApiBase/proxy" `
            -Method POST `
            -Headers $headers `
            -Body $proxyBody `
            -ErrorAction Stop
        
        Write-Host "✅ ALLOWED" -ForegroundColor Green
        
        # Check logs for region
        Start-Sleep -Seconds 1
        $logsUrl = "$ApiBase/logs?limit=1" + [char]0x26 + "agent_id=$AgentId"
        $logs = Invoke-RestMethod -Uri $logsUrl `
            -Method GET `
            -Headers @{ "Authorization" = "Bearer $token" } `
            -ErrorAction SilentlyContinue
        
        $detectedRegion = "Unknown"
        if ($logs.data -and $logs.data.Count -gt 0) {
            $log = $logs.data[0]
            if ($log.target_region) {
                $detectedRegion = $log.target_region
            }
        }
        
        Write-Host "   Detected Region: $detectedRegion" -ForegroundColor Cyan
        Write-Host "   Status: COMPLIANT" -ForegroundColor Green
        
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        
        if ($statusCode -eq 403) {
            # Request was blocked
            Write-Host "❌ ACCESS DENIED" -ForegroundColor Red
            
            try {
                $errorBody = $_.ErrorDetails.Message | ConvertFrom-Json
                
                # Check logs for region first (most reliable source)
                Start-Sleep -Seconds 1
                $logsUrl = "$ApiBase/logs?limit=1" + [char]0x26 + "agent_id=$AgentId"
                $logs = Invoke-RestMethod -Uri $logsUrl `
                    -Method GET `
                    -Headers @{ "Authorization" = "Bearer $token" } `
                    -ErrorAction SilentlyContinue
                
                $detectedRegion = $null
                if ($logs.data -and $logs.data.Count -gt 0) {
                    $log = $logs.data[0]
                    if ($log.target_region) {
                        $detectedRegion = $log.target_region
                    }
                }
                
                # Fallback to error response if logs don't have region
                if (-not $detectedRegion -and $errorBody.detected_country) {
                    $detectedRegion = $errorBody.detected_country
                }
                
                # If still no region, check for UNKNOWN in error response
                if (-not $detectedRegion) {
                    $detectedRegion = "Unknown"
                }
                
                Write-Host "   Detected Region: $detectedRegion" -ForegroundColor Yellow
                
                # Determine reason
                $reason = "Access blocked by Sovereign Lock"
                if ($errorBody.error -eq "SOVEREIGN_LOCK_VIOLATION") {
                    $reason = "Access blocked by Sovereign Lock"
                } elseif ($errorBody.error -eq "SOVEREIGNTY_CHECK_FAILED") {
                    $reason = "Could not verify data sovereignty"
                } elseif ($errorBody.status) {
                    $reason = "Status: $($errorBody.status)"
                }
                
                Write-Host "   Reason: $reason" -ForegroundColor Yellow
                
            } catch {
                Write-Host "   Detected Region: Unknown" -ForegroundColor Yellow
                Write-Host "   Reason: Access blocked by Sovereign Lock" -ForegroundColor Yellow
            }
        } else {
            Write-Host "❌ ERROR" -ForegroundColor Red
            Write-Host "   HTTP Status: $statusCode" -ForegroundColor White
            $errorMsg = $_.Exception.Message
            Write-Host "   Error: $errorMsg" -ForegroundColor Red
        }
    }
    
    Write-Host ""
}

# Export function if script is being sourced
if ($MyInvocation.InvocationName -ne '.') {
    Export-ModuleMember -Function Test-Proxy
}

