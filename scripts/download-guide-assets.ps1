<#
.SYNOPSIS
    Downloads texture/model assets referenced in docs/free-texture-sources-guide.md.
.DESCRIPTION
    Downloads textures from ambientCG (direct ZIPs) and helps with 3DTextures.me (Google Drive).
    For 3DTextures.me assets, the script will display Google Drive folder links.
    Download these manually and the script will process them automatically on next run.
    Converts JPG to PNG so the engine can load expected *.png filenames.
.NOTES
    Run from project root: .\scripts\download-guide-assets.ps1
    For Google Drive downloads, save the ZIPs to temp/guide-downloads/gdrive/ folder.
#>

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$TempRoot = "temp/guide-downloads"

function Ensure-Dir {
    param([string]$Path)
    if (!(Test-Path $Path)) {
        New-Item -ItemType Directory -Force -Path $Path | Out-Null
    }
}

function Download-File {
    param(
        [string]$Url,
        [string]$Dest
    )
    if ((Test-Path $Dest) -and -not $Force) {
        Write-Host "Skipping existing: $Dest"
        return $true
    }
    Write-Host "Downloading: $Url"
    try {
        Ensure-Dir (Split-Path $Dest -Parent)
        $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" }
        Invoke-WebRequest -Uri $Url -OutFile $Dest -Headers $headers -UseBasicParsing -MaximumRedirection 5 -ErrorAction Stop
        return $true
    }
    catch {
        Write-Warning "Failed: $Url"
        return $false
    }
}

function Expand-ZipSafe {
    param(
        [string]$ZipPath,
        [string]$DestDir
    )
    if (!(Test-Path $ZipPath)) {
        Write-Warning "ZIP not found: $ZipPath"
        return $false
    }
    if (Test-Path $DestDir) {
        Remove-Item -Recurse -Force $DestDir
    }
    Ensure-Dir $DestDir
    Expand-Archive -Path $ZipPath -DestinationPath $DestDir -Force
    return $true
}

function Convert-ToPng {
    param(
        [string]$SourcePath,
        [string]$DestPath
    )
    try {
        Add-Type -AssemblyName System.Drawing
        $image = [System.Drawing.Image]::FromFile($SourcePath)
        $image.Save($DestPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $image.Dispose()
        return $true
    }
    catch {
        Write-Warning "PNG conversion failed: $SourcePath"
        return $false
    }
}

function Copy-ToPng {
    param(
        [string]$SourcePath,
        [string]$DestPath
    )
    $ext = [IO.Path]::GetExtension($SourcePath).ToLowerInvariant()
    if ($ext -eq ".png") {
        Copy-Item -Path $SourcePath -Destination $DestPath -Force
        return $true
    }
    return (Convert-ToPng -SourcePath $SourcePath -DestPath $DestPath)
}

function Find-FirstMatch {
    param(
        [string]$Root,
        [string]$Pattern
    )
    return Get-ChildItem -Path $Root -Recurse -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like $Pattern } |
        Select-Object -First 1
}

function Find-ZipLink {
    param(
        [string]$PageUrl,
        [string]$ContainsText
    )
    try {
        $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" }
        $resp = Invoke-WebRequest -Uri $PageUrl -Headers $headers -UseBasicParsing -ErrorAction Stop
        foreach ($link in $resp.Links) {
            if ($link.href -match "\.zip$") {
                if ([string]::IsNullOrWhiteSpace($ContainsText) -or $link.href -like "*$ContainsText*") {
                    return $link.href
                }
            }
        }
    }
    catch {
        Write-Warning "Failed to fetch page: $PageUrl"
    }
    return $null
}

function Download-GoogleDriveFolder {
    param(
        [string]$FolderId,
        [string]$ZipName,
        [string]$DestDir,
        [hashtable]$MapPatterns
    )
    $zipPath = Join-Path $TempRoot "gdrive/$ZipName.zip"
    $extractDir = Join-Path $TempRoot "gdrive/$ZipName"

    # Google Drive download URL for folders (downloads as zip)
    $url = "https://drive.google.com/drive/folders/$FolderId"

    Write-Host "Google Drive folder: $ZipName"
    Write-Host "NOTE: Google Drive folders cannot be automatically downloaded."
    Write-Host "Please manually download from: $url"
    Write-Host "Save to: $zipPath"

    # Check if user has already downloaded the file
    if (Test-Path $zipPath) {
        Write-Host "Found existing download: $zipPath"
        if (-not (Expand-ZipSafe -ZipPath $zipPath -DestDir $extractDir)) { return }

        Ensure-Dir $DestDir
        foreach ($key in $MapPatterns.Keys) {
            $pattern = $MapPatterns[$key]
            $match = Find-FirstMatch -Root $extractDir -Pattern $pattern
            if ($null -eq $match) {
                Write-Warning "No match for $key in $ZipName ($pattern)"
                continue
            }
            $destPath = Join-Path $DestDir $key
            Copy-ToPng -SourcePath $match.FullName -DestPath $destPath | Out-Null
            Write-Host "Saved: $destPath"
        }
    }
    else {
        Write-Host ""
    }
}

function Fetch-AmbientCg {
    param(
        [string]$FileId,
        [string]$DestDir,
        [hashtable]$MapPatterns
    )
    $zipName = "$FileId.zip"
    $zipPath = Join-Path $TempRoot "ambientcg/$zipName"
    $extractDir = Join-Path $TempRoot "ambientcg/$FileId"
    $url = "https://ambientcg.com/get?file=$FileId.zip"

    if (-not (Download-File -Url $url -Dest $zipPath)) {
        $fallback = Find-ZipLink -PageUrl "https://ambientcg.com/view?id=$($FileId.Split('_')[0])" -ContainsText $FileId
        if (-not $fallback) { return }
        if (-not (Download-File -Url $fallback -Dest $zipPath)) { return }
    }
    if (-not (Expand-ZipSafe -ZipPath $zipPath -DestDir $extractDir)) { return }

    Ensure-Dir $DestDir

    foreach ($key in $MapPatterns.Keys) {
        $pattern = $MapPatterns[$key]
        $match = Find-FirstMatch -Root $extractDir -Pattern $pattern
        if ($null -eq $match) {
            Write-Warning "No match for $key in $FileId ($pattern)"
            continue
        }
        $destPath = Join-Path $DestDir $key
        Copy-ToPng -SourcePath $match.FullName -DestPath $destPath | Out-Null
        Write-Host "Saved: $destPath"
    }
}

function Fetch-3dTextures {
    param(
        [string[]]$CandidateUrls,
        [string]$ZipName,
        [string]$DestDir,
        [string]$SourcePageUrl,
        [hashtable]$MapPatterns
    )
    $zipPath = Join-Path $TempRoot "3dtextures/$ZipName.zip"
    $extractDir = Join-Path $TempRoot "3dtextures/$ZipName"

    $downloaded = $false
    foreach ($url in $CandidateUrls) {
        if (Download-File -Url $url -Dest $zipPath) {
            $downloaded = $true
            break
        }
    }
    if (-not $downloaded -and $SourcePageUrl) {
        $pageZip = Find-ZipLink -PageUrl $SourcePageUrl -ContainsText $ZipName
        if ($pageZip -and (Download-File -Url $pageZip -Dest $zipPath)) {
            $downloaded = $true
        }
    }
    if (-not $downloaded) {
        Write-Warning "Failed to download: $ZipName"
        return
    }

    if (-not (Expand-ZipSafe -ZipPath $zipPath -DestDir $extractDir)) { return }
    Ensure-Dir $DestDir

    foreach ($key in $MapPatterns.Keys) {
        $pattern = $MapPatterns[$key]
        $match = Find-FirstMatch -Root $extractDir -Pattern $pattern
        if ($null -eq $match) {
            Write-Warning "No match for $key in $ZipName ($pattern)"
            continue
        }
        $destPath = Join-Path $DestDir $key
        Copy-ToPng -SourcePath $match.FullName -DestPath $destPath | Out-Null
        Write-Host "Saved: $destPath"
    }
}

function Try-Download-CropsBundle {
    # Try multiple potential URLs for the Ultimate Crops Pack
    $bundleUrls = @(
        "https://poly.pizza/m/Ro6K0Yg7mx",
        "https://quaternius.com/packs/ultimatecrops.html"
    )
    $zipPath = Join-Path $TempRoot "crops/ultimate-crops.zip"
    $extractDir = Join-Path $TempRoot "crops/ultimate-crops"
    $destRoot = "assets/models/crops"

    Write-Host "Checking crops bundle pages..."

    $zipLink = $null
    foreach ($bundleUrl in $bundleUrls) {
        try {
            $headers = @{ "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" }
            $resp = Invoke-WebRequest -Uri $bundleUrl -Headers $headers -UseBasicParsing -ErrorAction Stop

            foreach ($link in $resp.Links) {
                if ($link.href -match "\.zip$") {
                    $zipLink = $link.href
                    Write-Host "Found ZIP link at: $bundleUrl"
                    break
                }
            }
            if ($zipLink) { break }
        }
        catch {
            Write-Warning "Failed to fetch: $bundleUrl"
        }
    }

    if (-not $zipLink) {
        Write-Warning "No direct ZIP link found for crops bundle."
        Write-Warning "Manual download may be required from:"
        Write-Warning "  - https://poly.pizza/m/Ro6K0Yg7mx"
        Write-Warning "  - https://quaternius.com/packs/ultimatecrops.html"
        return
    }

    if (-not (Download-File -Url $zipLink -Dest $zipPath)) { return }
    if (-not (Expand-ZipSafe -ZipPath $zipPath -DestDir $extractDir)) { return }

    Ensure-Dir $destRoot
    $glbFiles = Get-ChildItem -Path $extractDir -Recurse -Filter "*.glb" -ErrorAction SilentlyContinue
    foreach ($file in $glbFiles) {
        # Best-effort placement based on filename keywords
        if ($file.Name -match "wheat") {
            Ensure-Dir "$destRoot/wheat"
            Copy-Item $file.FullName "$destRoot/wheat/$($file.Name)" -Force
        }
        elseif ($file.Name -match "carrot") {
            Ensure-Dir "$destRoot/carrot"
            Copy-Item $file.FullName "$destRoot/carrot/$($file.Name)" -Force
        }
        elseif ($file.Name -match "corn") {
            Ensure-Dir "$destRoot/corn"
            Copy-Item $file.FullName "$destRoot/corn/$($file.Name)" -Force
        }
    }
    Write-Host "Crops models copied (best-effort)."
}

Ensure-Dir $TempRoot

# Folder structure from the guide
$folders = @(
    "assets/pbr/buildings/wood_plank",
    "assets/pbr/buildings/stone_brick",
    "assets/pbr/buildings/metal_plate",
    "assets/pbr/buildings/thatch",
    "assets/pbr/buildings/wood_shingles",
    "assets/pbr/terrain/grass",
    "assets/pbr/terrain/dirt",
    "assets/pbr/terrain/rock",
    "assets/pbr/terrain/sand",
    "assets/pbr/terrain/tilled_soil",
    "assets/pbr/props/rocks/rock_large",
    "assets/pbr/props/cobblestone",
    "assets/pbr/props/containers/crate",
    "assets/pbr/props/containers/barrel",
    "assets/pbr/water",
    "assets/models/crops/wheat",
    "assets/models/crops/carrot",
    "assets/models/crops/corn"
)
foreach ($folder in $folders) { Ensure-Dir $folder }

# ambientCG (terrain + water)
Fetch-AmbientCg -FileId "Grass001_1K-JPG" -DestDir "assets/pbr/terrain/grass" -MapPatterns @{
    "albedo.png" = "*_Color.*";
    "normal.png" = "*_NormalGL.*";
}
Fetch-AmbientCg -FileId "Ground037_1K-JPG" -DestDir "assets/pbr/terrain/dirt" -MapPatterns @{
    "albedo.png" = "*_Color.*";
    "normal.png" = "*_NormalGL.*";
}
Fetch-AmbientCg -FileId "Rock030_1K-JPG" -DestDir "assets/pbr/terrain/rock" -MapPatterns @{
    "albedo.png" = "*_Color.*";
    "normal.png" = "*_NormalGL.*";
}
Fetch-AmbientCg -FileId "Ground054_1K-JPG" -DestDir "assets/pbr/terrain/sand" -MapPatterns @{
    "albedo.png" = "*_Color.*";
    "normal.png" = "*_NormalGL.*";
}
Fetch-AmbientCg -FileId "Ground048_1K-JPG" -DestDir "assets/pbr/terrain/tilled_soil" -MapPatterns @{
    "albedo.png" = "*_Color.*";
    "normal.png" = "*_NormalGL.*";
}
Fetch-AmbientCg -FileId "Water002_1K-JPG" -DestDir "assets/pbr/water" -MapPatterns @{
    "flow_normal.png" = "*_NormalGL.*";
}

# 3DTextures.me (buildings + props) - Google Drive downloads
Download-GoogleDriveFolder -FolderId "1mKBGerzy1BKQvjCkyMV2BmfAoIdXnWQh" -ZipName "Stylized_Wood_Wall_001" -DestDir "assets/pbr/buildings/wood_plank" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1QGWGddAUA1m7bBxpp-mM32hYUbABbv_H" -ZipName "Stylized_Stone_Wall_001" -DestDir "assets/pbr/buildings/stone_brick" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1zAczmGVyzWK7PuF9D93yUzWlchCHjCrZ" -ZipName "Stylized_Metal_Plates_001" -DestDir "assets/pbr/buildings/metal_plate" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "metallic.png" = "*_metallic.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1L20U-CXcEG-2zK2a5Ybe0SP7HR1gnk9o" -ZipName "Thatched_Roof_001" -DestDir "assets/pbr/buildings/thatch" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1TsVtvlSzuQVTIOm6vDpzKGaSmQUko21o" -ZipName "Stylized_Wood_Shingles_001" -DestDir "assets/pbr/buildings/wood_shingles" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1wwM0_OpPZlebhKD1mRwGjMXu6fNk1BHL" -ZipName "Stylized_Cliff_Rock_001" -DestDir "assets/pbr/props/rocks/rock_large" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1pgRNS3ZAgTZyDb0w6BzWyW9ZNYha6FTH" -ZipName "Cobblestone_Irregular_Floor_001" -DestDir "assets/pbr/props/cobblestone" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
    "roughness.png" = "*_roughness.*";
    "ao.png" = "*_ambientOcclusion.*";
}
Download-GoogleDriveFolder -FolderId "1aL5FywCnBOxLbUWI0JWubh23WwQ4lBkr" -ZipName "Stylized_Crate_001" -DestDir "assets/pbr/props/containers/crate" -MapPatterns @{
    "albedo.png" = "*_basecolor.*";
    "normal.png" = "*_normal.*";
}

# Crops bundle (best-effort)
Try-Download-CropsBundle

Write-Host ""
Write-Host "Done. If any downloads failed, re-run with -Force after providing direct ZIP URLs."
