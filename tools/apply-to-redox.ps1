# Aplica overlay Redox AIOS sobre o tree Redox upstream.

param(
    [Parameter(Mandatory = $false)]
    [string]$RedoxRoot = (Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "redox")
)

$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot

if (-not (Test-Path $RedoxRoot)) {
    Write-Error "Redox root nao encontrado: $RedoxRoot"
}

function Copy-Tree($Source, $Dest) {
    if (-not (Test-Path $Source)) { return }
    New-Item -ItemType Directory -Force -Path $Dest | Out-Null
    Copy-Item -Path (Join-Path $Source "*") -Destination $Dest -Recurse -Force
}

Write-Host "Aplicando overlay AIOS..."
Write-Host "  De:   $AiosRoot"
Write-Host "  Para: $RedoxRoot"

# Config
Copy-Tree (Join-Path $AiosRoot "config") (Join-Path $RedoxRoot "config")

# Recipes
Copy-Tree (Join-Path $AiosRoot "recipes\aios") (Join-Path $RedoxRoot "recipes\aios")
Copy-Tree (Join-Path $AiosRoot "recipes\groups\aios") (Join-Path $RedoxRoot "recipes\groups\aios")
Copy-Tree (Join-Path $AiosRoot "recipes\groups\aios-skills-staging") (Join-Path $RedoxRoot "recipes\groups\aios-skills-staging")

# Tools (guest scripts, demos)
Copy-Tree (Join-Path $AiosRoot "tools") (Join-Path $RedoxRoot "tools\aios")

# Crates (necessario para path= nos recipes)
Copy-Tree (Join-Path $AiosRoot "crates") (Join-Path $RedoxRoot "crates")

# Docs (opcional, nao sobrescreve upstream)
$docsDest = Join-Path $RedoxRoot "docs\aios"
Copy-Tree (Join-Path $AiosRoot "docs") $docsDest

# Makefile target hint
$mkPath = Join-Path $RedoxRoot "mk\aios.mk"
Copy-Item -Path (Join-Path $AiosRoot "mk\aios.mk") -Destination $mkPath -Force

# Incluir no Makefile principal se ainda nao estiver
$makefile = Join-Path $RedoxRoot "Makefile"
if (Test-Path $makefile) {
    $content = Get-Content $makefile -Raw
    if ($content -notmatch "aios\.mk") {
        Add-Content -Path $makefile -Value "`ninclude mk/aios.mk`n"
    }
}

Write-Host "Overlay aplicado com sucesso."
