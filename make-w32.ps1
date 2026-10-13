$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location -LiteralPath $PSScriptRoot
$env:PATH = "${PWD}\etc\tools\gnu-make\win32\bin;${env:PATH}"
& make @args
exit $LASTEXITCODE
