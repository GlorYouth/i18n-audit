# �ű�����һ����ѡ����������ָ�������ʽ
param (
    [string]$Format = "text"
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$testDirs = Get-ChildItem -Path $PSScriptRoot -Directory -Filter "mini_test_*"

# ������Ŀ
Write-Host "���ڱ��� i18n-audit..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

foreach ($testDir in $testDirs) {
    $testName = $testDir.Name
    Write-Host "`n--- ��ʼ����: $testName ---" -ForegroundColor Magenta

    # ����ʾ������
    Write-Host "`n--- ���з���ʾ������ ($testName) ---" -ForegroundColor Green
    Set-Location -Path $testDir.FullName
    cargo run
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    # ������ƹ���
    Write-Host "`n--- ���� i18n-audit (��ʽ: $Format, ����: $testName) ---" -ForegroundColor Green
    Set-Location -Path $PSScriptRoot
    & "$projectRoot\target\release\i18n-audit.exe" -p $testDir.FullName --verbose run -f $Format
    
    Write-Host "`n--- �������: $testName ---" -ForegroundColor Magenta
}

Write-Host "`n--- ���в�������� ---" -ForegroundColor Cyan
# ���ص�ԭʼĿ¼
Set-Location -Path $projectRoot 