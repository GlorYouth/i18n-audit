# 脚本接受一个可选参数，用于指定输出格式
param (
    [string]$Format = "text"
)

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
Write-Host "`n--- 运行 i18n-audit (格式: $Format) ---" -ForegroundColor Green
Set-Location -Path $PSScriptRoot
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" --verbose run -f $Format

Write-Host "`n--- 测试完成 ---" -ForegroundColor Magenta
# 返回到原始目录
Set-Location -Path $projectRoot 