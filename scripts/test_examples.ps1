# Regression Test Script for WTLang Examples
# This script runs all examples through the compiler to ensure they still work

$EXAMPLES_DIR = "examples"
$OUTPUT_DIR = "test_regression_output"
$WTC = ".\target\release\wtc.exe"

# Colors for output
function Write-Success { param($msg) Write-Host $msg -ForegroundColor Green }
function Write-Error { param($msg) Write-Host $msg -ForegroundColor Red }
function Write-Info { param($msg) Write-Host $msg -ForegroundColor Cyan }

# Create output directory
if (Test-Path $OUTPUT_DIR) {
    Remove-Item -Recurse -Force $OUTPUT_DIR
}
New-Item -ItemType Directory -Path $OUTPUT_DIR | Out-Null

# Build the compiler first
Write-Info "Building compiler..."
cargo build --release -p wtlang-compiler
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build compiler"
    exit 1
}

# Get all .wt files in examples directory
$examples = Get-ChildItem -Path $EXAMPLES_DIR -Filter "*.wt"
$total = $examples.Count
$passed = 0
$failed = 0

Write-Info "Running $total example(s)...`n"

foreach ($example in $examples) {
    $basename = $example.BaseName
    Write-Info "Testing $($example.Name)..."
    
    # Check syntax
    & $WTC check $example.FullName
    if ($LASTEXITCODE -ne 0) {
        Write-Error "  FAIL: Syntax check failed"
        $failed++
        continue
    }
    
    # Compile
    $outputPath = Join-Path $OUTPUT_DIR $basename
    & $WTC build $example.FullName --output $outputPath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "  FAIL: Compilation failed"
        $failed++
        continue
    }
    
    # Verify Python syntax (if Python is available)
    if (Get-Command python -ErrorAction SilentlyContinue) {
        $pyFiles = Get-ChildItem -Path $outputPath -Filter "*.py"
        $pyValid = $true
        foreach ($pyFile in $pyFiles) {
            python -m py_compile $pyFile.FullName 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) {
                Write-Error "  FAIL: Generated Python is invalid ($($pyFile.Name))"
                $pyValid = $false
                break
            }
        }
        if (-not $pyValid) {
            $failed++
            continue
        }
    }
    
    Write-Success "  PASS"
    $passed++
}

# Cleanup
Write-Info "`nCleaning up..."
Remove-Item -Recurse -Force $OUTPUT_DIR

# Summary
Write-Info "`n========================================="
Write-Info "Test Summary:"
Write-Info "  Total:  $total"
Write-Success "  Passed: $passed"
if ($failed -gt 0) {
    Write-Error "  Failed: $failed"
    exit 1
} else {
    Write-Success "All regression tests passed!"
    exit 0
}
