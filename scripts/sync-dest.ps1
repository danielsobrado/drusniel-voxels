[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'Low')]
param(
    [string]$RustDest = 'F:\\Development\\workspace\\GitHub\\drusniel-voxels',
    [string]$WebDest = 'F:\\Development\\workspace\\GitHub\\drusniel-voxels-web'
)

$ErrorActionPreference = 'Stop'

function Resolve-RepoRoot {
    $repoRoot = (& rtk git rev-parse --show-toplevel).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($repoRoot)) {
        throw 'Could not determine the repository root with git.'
    }
    return (Get-Item -LiteralPath $repoRoot).FullName
}

function Should-ExcludePath {
    param([string]$Path)
    if ($Path -like '*.md') { return $true }
    if ($Path -in @('AGENTS.md', 'CLAUDE.md')) { return $true }
    if ($Path -like 'docs/reference/*') { return $true }
    if ($Path -like 'node_modules/*') { return $true }
    return $false
}

function Copy-SourceFile {
    param(
        [string]$Source,
        [string]$Dest
    )
    $destDir = Split-Path -Parent $Dest
    if (-not (Test-Path -LiteralPath $destDir -PathType Container)) {
        if ($PSCmdlet.ShouldProcess($destDir, 'Create destination directory')) {
            New-Item -ItemType Directory -Force -Path $destDir | Out-Null
        }
    }
    if ($PSCmdlet.ShouldProcess($Source, "Copy to $Dest")) {
        Copy-Item -Force -LiteralPath $Source -Destination $Dest
    }
}

function Remove-DestFile {
    param([string]$Dest)
    if (Test-Path -LiteralPath $Dest -PathType Leaf) {
        if ($PSCmdlet.ShouldProcess($Dest, 'Remove destination file')) {
            Remove-Item -Force -LiteralPath $Dest
        }
    }
}

$SourceRoot = Resolve-RepoRoot
$RustDestRoot = (Get-Item -LiteralPath $RustDest).FullName
$WebDestRoot = (Get-Item -LiteralPath $WebDest).FullName

$StatusLines = & git status --porcelain -M --untracked-files=all 2>$null | ForEach-Object { $_.Trim() }
if ($LASTEXITCODE -ne 0) {
    throw 'Failed to read git status from the source repository.'
}

$Copied = @()
$Deleted = @()
$Renames = @()

foreach ($line in $StatusLines) {
    if ($line -match '^[R]\d*\s+(.+?)\s+->\s+(.+)$') {
        $Renames += @{ Old = $Matches[1]; New = $Matches[2] }
        continue
    }

    if ($line -match '^[ MADRCUT?]{2}\s+(.+)$') {
        $path = $Matches[1]
        if (Should-ExcludePath -Path $path) { continue }

        $sourceFile = Join-Path $SourceRoot $path
        $destRoot = if ($path -like 'tools/clod-poc/*') { $WebDestRoot } else { $RustDestRoot }
        $destRel = if ($path -like 'tools/clod-poc/*') { $path.Substring('tools/clod-poc/'.Length) } else { $path }
        $destFile = Join-Path $destRoot $destRel

        if (-not (Test-Path -LiteralPath $sourceFile -PathType Leaf)) {
            Remove-DestFile -Dest $destFile
            $Deleted += $destRel
            continue
        }

        Copy-SourceFile -Source $sourceFile -Dest $destFile
        $Copied += $destRel
    }
}

foreach ($rename in $Renames) {
    $oldPath = $rename.Old
    $newPath = $rename.New
    if (Should-ExcludePath -Path $oldPath -or Should-ExcludePath -Path $newPath) {
        continue
    }

    $oldDestRoot = if ($oldPath -like 'tools/clod-poc/*') { $WebDestRoot } else { $RustDestRoot }
    $newDestRoot = if ($newPath -like 'tools/clod-poc/*') { $WebDestRoot } else { $RustDestRoot }
    $oldDestRel = if ($oldPath -like 'tools/clod-poc/*') { $oldPath.Substring('tools/clod-poc/'.Length) } else { $oldPath }
    $newDestRel = if ($newPath -like 'tools/clod-poc/*') { $newPath.Substring('tools/clod-poc/'.Length) } else { $newPath }
    $oldDestFile = Join-Path $oldDestRoot $oldDestRel
    $newDestFile = Join-Path $newDestRoot $newDestRel

    Remove-DestFile -Dest $oldDestFile
    $Deleted += $oldDestRel

    $sourceFile = Join-Path $SourceRoot $newPath
    if (Test-Path -LiteralPath $sourceFile -PathType Leaf) {
        Copy-SourceFile -Source $sourceFile -Dest $newDestFile
        $Copied += $newDestRel
    }
}

Write-Host '---'
Write-Host 'Copied files:'
if ($Copied.Count -gt 0) { $Copied | Sort-Object | Get-Unique | ForEach-Object { Write-Host "  $_" } } else { Write-Host '  none' }
Write-Host 'Deleted files:'
if ($Deleted.Count -gt 0) { $Deleted | Sort-Object | Get-Unique | ForEach-Object { Write-Host "  $_" } } else { Write-Host '  none' }
Write-Host '---'

Write-Host 'Rust destination status:'
Push-Location -LiteralPath $RustDestRoot
try { & git status --short --untracked-files=all } finally { Pop-Location }
Write-Host '---'
Write-Host 'Web destination status:'
Push-Location -LiteralPath $WebDestRoot
try { & git status --short --untracked-files=all } finally { Pop-Location }
