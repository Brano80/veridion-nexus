# Test Script for Enhanced Module System
# Tests the new database migration and ModuleService extensions

Write-Host "🧪 Testing Enhanced Module System" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

$API_BASE = $env:API_URL
if (-not $API_BASE) {
    $API_BASE = "http://localhost:8080/api/v1"
}

Write-Host "API Base URL: $API_BASE" -ForegroundColor Yellow
Write-Host ""

# Test 1: Check if migration can be run (health check)
Write-Host "Test 1: Health Check" -ForegroundColor Green
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/../health" -Method Get -ErrorAction Stop
    Write-Host "✅ Health check passed: $($response.status)" -ForegroundColor Green
} catch {
    Write-Host "❌ Health check failed: $_" -ForegroundColor Red
    Write-Host "   Make sure the API server is running!" -ForegroundColor Yellow
    exit 1
}
Write-Host ""

# Test 2: Get all modules (should include new columns)
Write-Host "Test 2: Get All Modules (with new metadata)" -ForegroundColor Green
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/modules" -Method Get -ErrorAction Stop
    Write-Host "✅ Retrieved $($response.modules.Count) modules" -ForegroundColor Green
    
    if ($response.modules.Count -gt 0) {
        $firstModule = $response.modules[0]
        Write-Host "   Sample module: $($firstModule.name)" -ForegroundColor Gray
        Write-Host "   Display name: $($firstModule.display_name)" -ForegroundColor Gray
        Write-Host "   Category: $($firstModule.category)" -ForegroundColor Gray
        
        # Check if new fields exist (they might be null initially)
        if ($firstModule.PSObject.Properties.Name -contains "tier") {
            Write-Host "   ✅ New 'tier' field exists" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  'tier' field not found (migration may not be run)" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "❌ Failed to get modules: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Get modules by regulation (new endpoint - may not exist yet)
Write-Host "Test 3: Get Modules by Regulation (new endpoint)" -ForegroundColor Green
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/modules/by-regulation/GDPR" -Method Get -ErrorAction Stop
    Write-Host "✅ Retrieved modules for GDPR: $($response.Count) modules" -ForegroundColor Green
    foreach ($module in $response) {
        Write-Host "   - $($module.name): $($module.display_name)" -ForegroundColor Gray
    }
} catch {
    if ($_.Exception.Response.StatusCode -eq 404) {
        Write-Host "⚠️  Endpoint not found (needs to be implemented)" -ForegroundColor Yellow
        Write-Host "   This is expected - endpoint will be added in next step" -ForegroundColor Gray
    } else {
        Write-Host "❌ Failed: $_" -ForegroundColor Red
    }
}
Write-Host ""

# Test 4: Test ModuleService methods via existing endpoints
Write-Host "Test 4: Test Module Enable/Disable (existing functionality)" -ForegroundColor Green
try {
    # Get a module to test with
    $modulesResponse = Invoke-RestMethod -Uri "$API_BASE/modules" -Method Get -ErrorAction Stop
    if ($modulesResponse.modules.Count -gt 0) {
        $testModule = $modulesResponse.modules[0]
        $moduleName = $testModule.name
        
        Write-Host "   Testing with module: $moduleName" -ForegroundColor Gray
        
        # Check status
        $statusResponse = Invoke-RestMethod -Uri "$API_BASE/modules/$moduleName/status" -Method Get -ErrorAction Stop
        Write-Host "   Current status: enabled=$($statusResponse.enabled)" -ForegroundColor Gray
        
        # Try to enable (if not already enabled)
        if (-not $statusResponse.enabled) {
            $enableResponse = Invoke-RestMethod -Uri "$API_BASE/modules/$moduleName/enable" -Method Post -ErrorAction Stop
            Write-Host "   ✅ Module enabled successfully" -ForegroundColor Green
        } else {
            Write-Host "   ✅ Module already enabled" -ForegroundColor Green
        }
    }
} catch {
    Write-Host "❌ Failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 5: Database function test (via direct SQL if possible, or via API)
Write-Host "Test 5: Database Migration Verification" -ForegroundColor Green
Write-Host "   Checking if new database columns exist..." -ForegroundColor Gray
Write-Host "   (This requires database access - checking via API)" -ForegroundColor Gray

# We can't directly test database functions, but we can check if the API
# can handle the new functionality
Write-Host "   ⚠️  Direct database testing requires database access" -ForegroundColor Yellow
Write-Host "   Run this SQL to verify migration:" -ForegroundColor Yellow
Write-Host "   SELECT column_name FROM information_schema.columns" -ForegroundColor Cyan
Write-Host "   WHERE table_name = 'modules' AND column_name IN ('tier', 'regulation', 'article_number');" -ForegroundColor Cyan
Write-Host ""

# Test 6: Company module config (new functionality - may not have endpoint yet)
Write-Host "Test 6: Company Module Configuration (new endpoint)" -ForegroundColor Green
try {
    # First, we'd need a company_id - this is just a placeholder test
    Write-Host "   ⚠️  This endpoint needs to be implemented" -ForegroundColor Yellow
    Write-Host "   Expected: POST /api/v1/companies/{company_id}/modules/{module_name}/config" -ForegroundColor Gray
} catch {
    Write-Host "   ⚠️  Endpoint not yet implemented (expected)" -ForegroundColor Yellow
}
Write-Host ""

# Summary
Write-Host "=================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "✅ Migration 035 created: migrations/035_enhanced_module_system.sql" -ForegroundColor Green
Write-Host "✅ ModuleService extended: src/module_service.rs" -ForegroundColor Green
Write-Host "⏳ New API endpoints: Pending (next step)" -ForegroundColor Yellow
Write-Host "⏳ New modules: Pending (next step)" -ForegroundColor Yellow
Write-Host ""
Write-Host "📝 Next Steps:" -ForegroundColor Cyan
Write-Host "   1. Run migration: psql -d your_database -f migrations/035_enhanced_module_system.sql" -ForegroundColor White
Write-Host "   2. Restart API server to load new ModuleService methods" -ForegroundColor White
Write-Host "   3. Test new endpoints (once implemented)" -ForegroundColor White
Write-Host ""

