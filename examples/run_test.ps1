# �ű�����һ����ѡ����������ָ�������ʽ
param (
    [string]$Format = "text"
)

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
Write-Host "`n--- ���� i18n-audit (��ʽ: $Format) ---" -ForegroundColor Green
Set-Location -Path $PSScriptRoot
& "$projectRoot\target\release\i18n-audit.exe" -p "$PSScriptRoot\mini_test" --verbose run -f $Format

Write-Host "`n--- ������� ---" -ForegroundColor Magenta
# ���ص�ԭʼĿ¼
Set-Location -Path $projectRoot 