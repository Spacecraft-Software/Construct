#!/usr/bin/env pwsh

# Create the reference directory
$targetDir = "reference"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir
}

# Download the file
$url = "https://microsoft.github.io/rust-guidelines/agents/all.txt"
$sourceFile = "Rust-Guidelines.txt"
Write-Host "Downloading $url..."
Invoke-WebRequest -Uri $url -OutFile $sourceFile

# Read all lines
$lines = Get-Content $sourceFile

# Find the starting line (0-indexed) for each top-level section header.
# We match ONLY lines that start with "# " followed by a known section title,
# to avoid false matches from "# " inside code blocks.
$sectionHeaders = @(
    "# AI Guidelines",
    "# Application Guidelines",
    "# Documentation",
    "# FFI Guidelines",
    "# Library Guidelines",
    "# Performance Guidelines",
    "# Safety Guidelines",
    "# Universal Guidelines",
    "# Libraries / Building Guidelines",
    "# Libraries / Interoperability Guidelines",
    "# Libraries / Resilience Guidelines",
    "# Libraries / UX Guidelines"
)

$fileMap = [ordered]@{
    "# AI Guidelines"                           = "01_ai_guidelines.md"
    "# Application Guidelines"                  = "02_application_guidelines.md"
    "# Documentation"                           = "03_documentation.md"
    "# FFI Guidelines"                          = "04_ffi_guidelines.md"
    "# Library Guidelines"                      = "05_library_guidelines.md"
    "# Performance Guidelines"                  = "06_performance_guidelines.md"
    "# Safety Guidelines"                       = "07_safety_guidelines.md"
    "# Universal Guidelines"                    = "08_universal_guidelines.md"
    "# Libraries / Building Guidelines"         = "09_libraries_building_guidelines.md"
    "# Libraries / Interoperability Guidelines" = "10_libraries_interoperability_guidelines.md"
    "# Libraries / Resilience Guidelines"       = "11_libraries_resilience_guidelines.md"
    "# Libraries / UX Guidelines"               = "12_libraries_ux_guidelines.md"
}

# Locate the line index of each top-level section header
$sectionStarts = @()
for ($i = 0; $i -lt $lines.Count; $i++) {
    $trimmed = $lines[$i]
    foreach ($header in $sectionHeaders) {
        if ($trimmed -eq $header) {
            $sectionStarts += [PSCustomObject]@{ Header = $header; LineIndex = $i }
            break
        }
    }
}

Write-Host "Found $($sectionStarts.Count) sections."

# Write each section to its output file
Write-Host "Splitting $sourceFile..."
for ($s = 0; $s -lt $sectionStarts.Count; $s++) {
    $header   = $sectionStarts[$s].Header
    $startIdx = $sectionStarts[$s].LineIndex
    $endIdx   = if ($s + 1 -lt $sectionStarts.Count) { $sectionStarts[$s + 1].LineIndex - 1 } else { $lines.Count - 1 }

    $fileName = $fileMap[$header]
    $filePath = Join-Path $targetDir $fileName
    Write-Host "Writing $filePath (lines $($startIdx + 1)-$($endIdx + 1))..."

    $sectionLines = $lines[$startIdx..$endIdx]
    $sectionLines | Out-File -FilePath $filePath -Encoding utf8
}

Write-Host "Done."