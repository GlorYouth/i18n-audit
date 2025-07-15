# 脚本接受一个可选参数，用于指定输出格式
param (
    [string]$Format = "text"
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$testDirs = Get-ChildItem -Path $PSScriptRoot -Directory -Filter "mini_test_*"

# 编译项目
Write-Host "正在编译 i18n-audit..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

foreach ($testDir in $testDirs) {
    $testName = $testDir.Name
    Write-Host "`n--- 开始测试: $testName ---" -ForegroundColor Magenta

    # 运行示例程序
    Write-Host "`n--- 运行翻译示例程序 ($testName) ---" -ForegroundColor Green
    Set-Location -Path $testDir.FullName
    cargo run
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    # 运行审计工具
    Write-Host "`n--- 运行 i18n-audit (格式: $Format, 测试: $testName) ---" -ForegroundColor Green
    Set-Location -Path $PSScriptRoot
    & "$projectRoot\target\release\i18n-audit.exe" -p $testDir.FullName --verbose run -f $Format
    
    Write-Host "`n--- 测试完成: $testName ---" -ForegroundColor Magenta
}

Write-Host "`n--- 所有测试已完成 ---" -ForegroundColor Cyan
# 返回到原始目录
Set-Location -Path $projectRoot 