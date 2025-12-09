# Test-SovereignLock.ps1
# PowerShell function to test Sovereign Lock (Proxy Mode) functionality
# Usage: Test-SovereignLock "https://api.openai.com/v1/chat/completions"

function Test-SovereignLock {
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
    Write-Host "`n[INFO] Authenticating..." -ForegroundColor Cyan
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
        Write-Host "   [OK] Authentication successful" -ForegroundColor Green
    } catch {
        $errorMsg = $_.Exception.Message
        Write-Host "   [ERROR] Authentication failed: $errorMsg" -ForegroundColor Red
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
    
    Write-Host "`n[INFO] Testing Sovereign Lock for: $TargetUrl" -ForegroundColor Cyan
    Write-Host "   Agent ID: $AgentId" -ForegroundColor Gray
    Write-Host "   Method: $Method" -ForegroundColor Gray
    
    # Test proxy endpoint
    try {
        $response = Invoke-RestMethod -Uri "$ApiBase/proxy" `
            -Method POST `
            -Headers $headers `
            -Body $proxyBody `
            -ErrorAction Stop
        
        Write-Host "`n[ALLOWED] REQUEST ALLOWED" -ForegroundColor Green
        Write-Host "   Status: COMPLIANT" -ForegroundColor White
        Write-Host "   Response received from target server" -ForegroundColor White
        
        # Check logs for region
        Start-Sleep -Seconds 1
        $logsUrl = "$ApiBase/logs?limit=1" + [char]0x26 + "agent_id=$AgentId"
        $logs = Invoke-RestMethod -Uri $logsUrl `
            -Method GET `
            -Headers @{ "Authorization" = "Bearer $token" } `
            -ErrorAction SilentlyContinue
        
        if ($logs.data -and $logs.data.Count -gt 0) {
            $log = $logs.data[0]
            Write-Host "   Detected Region: $($log.target_region)" -ForegroundColor Cyan
        }
        
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        
        if ($statusCode -eq 403) {
            # Request was blocked
            try {
                $errorBody = $_.ErrorDetails.Message | ConvertFrom-Json
                
                Write-Host "`n[BLOCKED] REQUEST BLOCKED (Access Denied)" -ForegroundColor Red
                Write-Host "   Status: $($errorBody.status)" -ForegroundColor White
                Write-Host "   Error: $($errorBody.error)" -ForegroundColor White
                Write-Host "   Message: $($errorBody.message)" -ForegroundColor White
                
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
                
                if ($detectedRegion) {
                    Write-Host "   Detected Region: $detectedRegion" -ForegroundColor Yellow
                } else {
                    Write-Host "   Detected Region: Unknown" -ForegroundColor Yellow
                }
                
            } catch {
                Write-Host "`n[BLOCKED] REQUEST BLOCKED (Access Denied)" -ForegroundColor Red
                Write-Host "   HTTP Status: 403 Forbidden" -ForegroundColor White
                Write-Host "   Could not parse error response" -ForegroundColor Yellow
            }
        } else {
            Write-Host "`n[ERROR] Request failed" -ForegroundColor Red
            Write-Host "   HTTP Status: $statusCode" -ForegroundColor White
            $errorMsg = $_.Exception.Message
            Write-Host "   Error: $errorMsg" -ForegroundColor Red
        }
    }
    
    Write-Host ""
}

# Export function if script is being sourced
if ($MyInvocation.InvocationName -ne '.') {
    Export-ModuleMember -Function Test-SovereignLock
}
