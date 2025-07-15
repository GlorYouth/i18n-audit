# 编译项目
Write-Host "正在编译 i18n-audit..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# 运行示例程序
Write-Host "`n--- 运行翻译示例程序 ---" -ForegroundColor Green
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -Path "$PSScriptRoot\mini_test"
cargo run
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# 运行审计工具
Write-Host "`n--- 运行 i18n-audit ---" -ForegroundColor Green
Set-Location -Path $PSScriptRoot
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" --verbose

Write-Host "`n--- 以 JSON 格式输出 ---" -ForegroundColor Yellow
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" run -f json

Write-Host "`n--- 以 YAML 格式输出 ---" -ForegroundColor Yellow
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" run -f yaml

Write-Host "`n--- 测试完成 ---" -ForegroundColor Magenta
# 返回到原始目录
Set-Location -Path $projectRoot 