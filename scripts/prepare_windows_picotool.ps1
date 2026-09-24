param(
    [Parameter(Mandatory)][ValidateSet('x86', 'x64')][string]$Arch,
    [Parameter(Mandatory)][string]$WorkDir
)
$ErrorActionPreference = 'Stop'
$WorkDir = [IO.Path]::GetFullPath($WorkDir)
$toolDir = Join-Path $WorkDir 'tool'
New-Item -ItemType Directory -Force -Path $toolDir | Out-Null

function Download-Checked([string]$Url, [string]$Name, [string]$Hash) {
    $destination = Join-Path $WorkDir $Name
    if (!(Test-Path -LiteralPath $destination)) {
        Invoke-WebRequest $Url -OutFile $destination
    }
    if ((Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash -ne $Hash) {
        throw "Download checksum mismatch: $Name"
    }
    return $destination
}

function Run-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program exited with $LASTEXITCODE" }
}

if ($Arch -eq 'x64') {
    $archive = Download-Checked 'https://github.com/raspberrypi/pico-sdk-tools/releases/download/v2.3.1-0/picotool-2.3.1-x64-win.zip' 'picotool.zip' '68730BE0813F8F35BE2CCA147CF7F1572662D5DCF0F5BA468E02A6DD9E85DB2B'
    Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $WorkDir 'official') -Force
    $executables = @(Get-ChildItem -LiteralPath (Join-Path $WorkDir 'official') -Filter picotool.exe -Recurse -File)
    if ($executables.Count -ne 1) { throw 'Expected one picotool executable' }
    Copy-Item -LiteralPath $executables[0].FullName -Destination $toolDir
} else {
    # Run from an x86 MSVC developer shell; official prebuilt picotool is x64 only.
    $source = Join-Path $WorkDir 'picotool-src'
    $sdk = Join-Path $WorkDir 'pico-sdk'
    if (!(Test-Path -LiteralPath $source)) {
        Run-Checked git @('clone', '--depth', '1', '--branch', '2.3.1', 'https://github.com/raspberrypi/picotool.git', $source)
    }
    if (!(Test-Path -LiteralPath $sdk)) {
        Run-Checked git @('clone', '--depth', '1', '--branch', '2.3.1', 'https://github.com/raspberrypi/pico-sdk.git', $sdk)
    }
    $mbedtlsArchive = Download-Checked 'https://github.com/Mbed-TLS/mbedtls/releases/download/mbedtls-3.6.2/mbedtls-3.6.2.tar.bz2' 'mbedtls.tar.bz2' '8B54FB9BCF4D5A7078028E0520ACDDEFB7900B3E66FEC7F7175BB5B7D85CCDCA'
    Run-Checked tar @('-xf', $mbedtlsArchive, '-C', $WorkDir)
    $usbArchive = Download-Checked 'https://github.com/libusb/libusb/releases/download/v1.0.29/libusb-1.0.29.7z' 'libusb.7z' '964A38152CA9A104CD00EC8D2F0617B89CD814F9B635E29763C68563D951521D'
    $usb = Join-Path $WorkDir 'libusb'
    New-Item -ItemType Directory -Force -Path $usb | Out-Null
    Run-Checked tar @('-xf', $usbArchive, '-C', $usb)
    $build = Join-Path $WorkDir 'build'
    Run-Checked python @('-m', 'cmake', '-S', $source, '-B', $build, '-G', 'Ninja',
        "-DPICO_SDK_PATH=$sdk", "-DPICO_MBEDTLS_PATH=$WorkDir/mbedtls-3.6.2",
        "-DLIBUSB_INCLUDE_DIR=$usb/include", "-DLIBUSB_LIBRARIES=$usb/VS2022/MS32/dll/libusb-1.0.lib",
        '-DPICOTOOL_LIBUSB_ALLOW_DLL=ON', '-DPICOTOOL_CODE_OTP=0',
        '-DCMAKE_POLICY_DEFAULT_CMP0091=NEW', '-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded', '-DCMAKE_BUILD_TYPE=Release')
    Run-Checked python @('-m', 'cmake', '--build', $build, '--parallel', '4')
    Copy-Item -LiteralPath "$build/picotool.exe", "$usb/VS2022/MS32/dll/libusb-1.0.dll" -Destination $toolDir
}
Run-Checked "$toolDir/picotool.exe" @('version')
Write-Output "picotool: $toolDir/picotool.exe"
