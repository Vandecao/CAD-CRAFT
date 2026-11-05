#requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repository = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$scriptPath = Join-Path $repository 'scripts/check-repository.ps1'
if (-not (Test-Path -LiteralPath $scriptPath)) {
    throw 'FALHA ESPERADA: script de diagnóstico ainda não implementado.'
}
. $scriptPath
$script:passed = 0
$script:failed = 0
function Assert-True($Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Test-Case([string]$Name, [scriptblock]$Body) {
    try {
        & $Body
        $script:passed++
        Write-Host "OK: $Name"
    } catch {
        $script:failed++
        Write-Host "FALHOU: $Name - $($_.Exception.Message)"
    }
}

Test-Case 'limites decimais de 50 e 100 MB sem arredondamento' {
    foreach ($case in @(
        @(50000000L, ''), @(50000001L, 'LARGE_FILE'),
        @(100000000L, 'LARGE_FILE'), @(100000001L, 'OVERSIZE_FILE')
    )) {
        $findings = @(Get-FilePolicyFindings -Path 'assets/model.bin' -IndexBytes $case[0])
        Assert-True (($findings.Code -join ',') -eq $case[1]) "Limite incorreto: $($case[0])"
    }
}
Test-Case 'tamanho maior do índice preservado apesar de arquivo local menor' {
    $findings = @(Get-FilePolicyFindings -Path 'assets/model.bin' -IndexBytes 100000001 -WorkingBytes 10)
    Assert-True ($findings.Code -contains 'OVERSIZE_FILE') 'Objeto grande do índice omitido.'
}
Test-Case 'arquivo de trabalho que cresceu também é alertado' {
    $findings = @(Get-FilePolicyFindings -Path 'assets/model.bin' -IndexBytes 10 -WorkingBytes 50000001)
    Assert-True ($findings.Code -contains 'LARGE_FILE') 'Arquivo local grande omitido.'
}
Test-Case 'temporários rastreados em diretórios aninhados' {
    foreach ($path in @('crates/cad-gui/target/debug/x', 'logs/run.log', 'tmp/x.bak', '.idea/x', 'vm/disk.vhdx')) {
        Assert-True (@(Get-FilePolicyFindings -Path $path).Code -contains 'TEMP_TRACKED') "Não identificado: $path"
    }
}
Test-Case 'sensíveis por nome sem expor conteúdo; exemplo e chave pública permitidos' {
    foreach ($path in @('.env', 'config/.env.production', 'private/key.pem', '.ssh/id_ed25519', '.aws/credentials')) {
        Assert-True (@(Get-FilePolicyFindings -Path $path).Code -contains 'SENSITIVE_FILE') "Não identificado: $path"
    }
    foreach ($path in @('.env.example', 'config/.env.example', '.ssh/id_ed25519.pub', 'assets/symbols.dll')) {
        Assert-True (@(Get-FilePolicyFindings -Path $path).Count -eq 0) "Falso positivo: $path"
    }
}
Test-Case 'AGENTS sem seção técnica ou com seção vazia requer revisão' {
    $content = Get-Content -LiteralPath (Join-Path $repository 'AGENTS.md') -Raw
    Assert-True (@(Get-AgentsStructureFindings -Content $content).Count -eq 0) 'AGENTS real inválido.'
    $withoutLicensing = $content -replace '(?s)## Licensing and data.*?(?=## Documentation)', ''
    Assert-True (@(Get-AgentsStructureFindings -Content $withoutLicensing).Count -gt 0) 'Seção ausente não detectada.'
    $emptySection = $content -replace '(?s)(## Tests)\r?\n.*?(?=\r?\n## )', '$1'
    Assert-True (@(Get-AgentsStructureFindings -Content $emptySection).Count -gt 0) 'Seção vazia não detectada.'
}
Test-Case 'AGENTS.local exige as seções do fluxo do autor' {
    $sections = ($script:LocalAgentsHeadings | ForEach-Object { "## $_`nconteúdo`n" }) -join "`n"
    $local = "# Fluxo local`n`n$sections"
    Assert-True (@(Get-AgentsStructureFindings $local $script:LocalAgentsHeadings).Count -eq 0) 'Fluxo local válido recusado.'
    $withoutPolicy = $local -replace '(?s)## POLÍTICA DE GIT, GITHUB E BACKUP.*', ''
    Assert-True (@(Get-AgentsStructureFindings $withoutPolicy $script:LocalAgentsHeadings).Count -gt 0) 'Política ausente não detectada.'
    $real = Join-Path $repository 'AGENTS.local.md'
    if (Test-Path -LiteralPath $real) {
        Assert-True (@(Get-AgentsStructureFindings (Get-Content -LiteralPath $real -Raw) $script:LocalAgentsHeadings).Count -eq 0) 'AGENTS.local real inválido.'
    }
}
Test-Case 'comando modificador recusado antes de iniciar Git' {
    $rejected = $false
    try { Invoke-RepositoryGit -RepositoryPath $repository -Arguments @('fetch') | Out-Null }
    catch { $rejected = $true }
    Assert-True $rejected 'Wrapper aceitou comando fora das consultas permitidas.'
}

$script:realGit = ${function:Invoke-RepositoryGit}
$script:mode = ''
function Invoke-RepositoryGit {
    param([string]$RepositoryPath, [string[]]$Arguments, [string]$InputText = '')
    $key = $Arguments -join ' '
    if ($script:mode -eq 'git-failure' -and $Arguments[0] -eq 'ls-files') {
        return [pscustomobject]@{ ExitCode = 128; Output = '' }
    }
    if ($script:mode -eq 'no-upstream' -and $key -eq 'rev-parse --symbolic-full-name @{upstream}') {
        return [pscustomobject]@{ ExitCode = 128; Output = '' }
    }
    if ($script:mode -eq 'unborn' -and $key -eq 'rev-parse --verify HEAD') {
        return [pscustomobject]@{ ExitCode = 128; Output = '' }
    }
    if ($script:mode -eq 'detached' -and $Arguments[0] -eq 'symbolic-ref') {
        return [pscustomobject]@{ ExitCode = 1; Output = '' }
    }
    if ($script:mode -eq 'diverged' -and $Arguments[0] -eq 'rev-list') {
        return [pscustomobject]@{ ExitCode = 0; Output = "3`t2`n" }
    }
    if ($script:mode -eq 'index-file' -and $key -eq 'ls-files --stage -z') {
        return [pscustomobject]@{ ExitCode = 0; Output = "100644 $('a' * 40) 0`tassets/diagrama elétrico.bin`0" }
    }
    if ($script:mode -eq 'index-file' -and $Arguments[0] -eq 'cat-file') {
        return [pscustomobject]@{ ExitCode = 0; Output = "$('a' * 40) blob 100000001`n" }
    }
    if ($script:mode -eq 'ignore-missing' -and $Arguments[0] -eq 'check-ignore') {
        return [pscustomobject]@{ ExitCode = 1; Output = '' }
    }
    if ($script:mode -eq 'ignore-source' -and $Arguments[0] -eq 'check-ignore') {
        $real = & $script:realGit @PSBoundParameters
        return [pscustomobject]@{ ExitCode = 0; Output = $real.Output + "Cargo.lock`0crates/cad-craft/src/lib.rs`0" }
    }
    & $script:realGit @PSBoundParameters
}
Test-Case 'sem upstream não inventa contagem ou sincronização' {
    $script:mode = 'no-upstream'
    $result = Get-RepositoryDiagnostics -RepositoryPath $repository
    Assert-True ($null -eq $result.Ahead -and $null -eq $result.Upstream) 'Inventou upstream ou contagem.'
    Assert-True ($result.RemoteState -eq 'ESTADO REMOTO NÃO VERIFICADO') 'Alegou consulta remota.'
}
Test-Case 'sem commits ainda produz diagnóstico' {
    $script:mode = 'unborn'
    $result = Get-RepositoryDiagnostics -RepositoryPath $repository
    Assert-True ($null -eq $result.Head -and $null -eq $result.Ahead) 'Inventou HEAD ou contagem.'
}
Test-Case 'HEAD destacado é identificado' {
    $script:mode = 'detached'
    Assert-True ((Get-RepositoryDiagnostics -RepositoryPath $repository).Branch -eq '(HEAD destacado)') 'Branch inventada.'
}
Test-Case 'contagem ahead/behind usa apenas referência local e ordem correta' {
    $script:mode = 'diverged'
    $result = Get-RepositoryDiagnostics -RepositoryPath $repository
    Assert-True ($result.Ahead -eq 3 -and $result.Behind -eq 2) 'Contagem invertida ou perdida.'
    Assert-True ($result.RemoteState -eq 'ESTADO REMOTO NÃO VERIFICADO') 'Tracking confundido com remoto atual.'
}
Test-Case 'índice com espaços e Unicode e arquivo ausente no disco' {
    $script:mode = 'index-file'
    $result = Get-RepositoryDiagnostics -RepositoryPath $repository
    $finding = @($result.Findings | Where-Object Code -eq 'OVERSIZE_FILE')
    Assert-True ($finding.Count -eq 1 -and $finding[0].Path -eq 'assets/diagrama elétrico.bin') 'Caminho ou tamanho perdido.'
}
Test-Case 'falha de inventário Git não resulta em falso relatório limpo' {
    $script:mode = 'git-failure'
    $caught = $false
    try { Get-RepositoryDiagnostics -RepositoryPath $repository | Out-Null } catch { $caught = $true }
    Assert-True $caught 'Falha de leitura omitida.'
}
Test-Case 'ignore ausente e regra que oculta fonte são detectados' {
    $script:mode = 'ignore-missing'
    Assert-True ((Get-RepositoryDiagnostics -RepositoryPath $repository).Findings.Code -contains 'IGNORE_MISSING') 'Proteção ausente omitida.'
    $script:mode = 'ignore-source'
    Assert-True ((Get-RepositoryDiagnostics -RepositoryPath $repository).Findings.Code -contains 'IGNORE_REQUIRED') 'Fonte ignorada omitida.'
}
${function:Invoke-RepositoryGit} = $script:realGit

Test-Case 'integração offline preserva todos os arquivos de metadados Git' {
    $gitDir = (& git -C $repository rev-parse --absolute-git-dir).Trim()
    function Get-GitSnapshot {
        @(Get-ChildItem -LiteralPath $gitDir -Recurse -Force -File | Sort-Object FullName | ForEach-Object {
            "$($_.FullName):$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash)"
        }) -join "`n"
    }
    $before = Get-GitSnapshot
    $result = Get-RepositoryDiagnostics -RepositoryPath $repository
    $after = Get-GitSnapshot
    Assert-True ($before -ceq $after) 'Arquivos Git alterados pelo diagnóstico.'
    Assert-True ($result.Findings.Count -eq 0) 'Repositório real contém achados; revisar relatório.'
    Assert-True ($result.RemoteState -eq 'ESTADO REMOTO NÃO VERIFICADO') 'Estado remoto inválido.'
}
Write-Host "Testes operacionais: $script:passed aprovados; $script:failed falharam."
if ($script:failed -gt 0) { exit 1 }
