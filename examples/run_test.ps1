# ������Ŀ
Write-Host "���ڱ��� i18n-audit..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# ����ʾ������
Write-Host "`n--- ���з���ʾ������ ---" -ForegroundColor Green
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -Path "$PSScriptRoot\mini_test"
cargo run
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# ������ƹ���
Write-Host "`n--- ���� i18n-audit ---" -ForegroundColor Green
Set-Location -Path $PSScriptRoot
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" --verbose

Write-Host "`n--- �� JSON ��ʽ��� ---" -ForegroundColor Yellow
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" run -f json

Write-Host "`n--- �� YAML ��ʽ��� ---" -ForegroundColor Yellow
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" run -f yaml

Write-Host "`n--- ������� ---" -ForegroundColor Magenta
# ���ص�ԭʼĿ¼
Set-Location -Path $projectRoot 