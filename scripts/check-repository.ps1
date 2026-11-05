#requires -Version 7.0
[CmdletBinding()]
param([string]$RepositoryPath = (Split-Path $PSScriptRoot -Parent))

# Somente diagnóstico: nenhuma consulta de rede, escrita, limpeza ou correção.
function Invoke-RepositoryGit {
    param([string]$RepositoryPath, [string[]]$Arguments, [string]$InputText = '')
    $allowed = @('rev-parse', 'symbolic-ref', 'rev-list', 'ls-files', 'cat-file', 'check-ignore', 'count-objects')
    if ($Arguments.Count -eq 0 -or $Arguments[0] -notin $allowed) {
        throw 'Comando fora das consultas Git permitidas pelo diagnóstico.'
    }
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardInputEncoding = [System.Text.UTF8Encoding]::new($false)
    $start.StandardOutputEncoding = [System.Text.UTF8Encoding]::new($false)
    $start.Environment['GIT_OPTIONAL_LOCKS'] = '0'
    $start.Environment['GIT_NO_LAZY_FETCH'] = '1'
    $start.Environment['GIT_TERMINAL_PROMPT'] = '0'
    foreach ($argument in (@('--no-optional-locks', '-c', 'core.fsmonitor=false', '-C', $RepositoryPath) + $Arguments)) {
        $start.ArgumentList.Add($argument)
    }
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.StandardInput.Write($InputText)
        $process.StandardInput.Close()
        $process.WaitForExit()
        # Nunca reproduzir stderr: configurações/URLs poderiam conter credenciais.
        [void]$stderr.GetAwaiter().GetResult()
        [pscustomobject]@{ ExitCode = $process.ExitCode; Output = $stdout.GetAwaiter().GetResult() }
    } finally {
        $process.Dispose()
    }
}

function Get-FilePolicyFindings {
    param([string]$Path, [long]$IndexBytes = 0, [long]$WorkingBytes = 0, [bool]$Tracked = $true)
    $size = [Math]::Max($IndexBytes, $WorkingBytes)
    if ($size -gt 100000000) {
        [pscustomobject]@{ Code = 'OVERSIZE_FILE'; Path = $Path; Message = "Acima de 100 MB: $size bytes; não adicionar ao Git convencional." }
    } elseif ($size -gt 50000000) {
        [pscustomobject]@{ Code = 'LARGE_FILE'; Path = $Path; Message = "Acima de 50 MB: $size bytes; avaliar antes do commit." }
    }
    if ($Tracked -and $Path -match '(?i)(^|/)(target|\.local|\.vs|\.idea|__pycache__|backups)(/|$)|\.(pdb|ilk|obj|o|dmp|log|tmp|bak|swp|pyc|vmdk|vdi|vhd|vhdx|vmem|vmss|qcow2|iso)$') {
        [pscustomobject]@{ Code = 'TEMP_TRACKED'; Path = $Path; Message = 'Possível build, cache, temporário, backup ou imagem de VM rastreado; revisar manualmente.' }
    }
    $name = ($Path -split '/')[-1]
    if (($name -ne '.env.example' -and $name -match '^\.env($|\.)') -or
        $name -match '(?i)\.(pem|key|pfx|p12|keystore)$|^id_(rsa|dsa|ecdsa|ed25519)$|^(credentials|secrets)(\.json)?$') {
        [pscustomobject]@{ Code = 'SENSITIVE_FILE'; Path = $Path; Message = 'Nome potencialmente sensível; conteúdo não lido nem exibido.' }
    }
}

# AGENTS.md público: regras técnicas. AGENTS.local.md (fora do Git): fluxo do autor.
$script:PublicAgentsHeadings = @('Architecture', 'Validation', 'Tests', 'Code style and language',
    'Licensing and data', 'Documentation')
$script:LocalAgentsHeadings = @('Retomada seletiva obrigatória', 'Escopo e estados', 'Arquitetura obrigatória',
    'Validação antes da revisão do usuário', 'Git, segurança e continuidade', 'Encerramento de sessão',
    'POLÍTICA DE GIT, GITHUB E BACKUP')

function Get-AgentsStructureFindings {
    param([string]$Content, [string[]]$Headings = $script:PublicAgentsHeadings)
    if ($Content -notmatch '(?m)^# .+') { 'Título principal ausente.' }
    foreach ($heading in $Headings) {
        $pattern = '(?ms)^## ' + [regex]::Escape($heading) + '\r?\n(?<body>.*?)(?=^## |\z)'
        $section = [regex]::Match($Content, $pattern)
        if (-not $section.Success -or [string]::IsNullOrWhiteSpace($section.Groups['body'].Value)) {
            "Seção obrigatória ausente ou vazia: $heading"
        }
    }
}

function Get-RepositoryDiagnostics {
    param([string]$RepositoryPath)
    $rootResult = Invoke-RepositoryGit $RepositoryPath @('rev-parse', '--show-toplevel')
    if ($rootResult.ExitCode -ne 0) { throw 'Diretório não é uma árvore de trabalho Git acessível.' }
    $root = $rootResult.Output.TrimEnd("`r", "`n")
    $findings = [System.Collections.Generic.List[object]]::new()
    $branchResult = Invoke-RepositoryGit $root @('symbolic-ref', '--quiet', '--short', 'HEAD')
    if ($branchResult.ExitCode -notin @(0, 1)) { throw 'Não foi possível consultar a branch.' }
    $branch = if ($branchResult.ExitCode -eq 0) { $branchResult.Output.Trim() } else { '(HEAD destacado)' }
    $headResult = Invoke-RepositoryGit $root @('rev-parse', '--verify', 'HEAD')
    $head = if ($headResult.ExitCode -eq 0) { $headResult.Output.Trim() } else { $null }
    $upstreamResult = Invoke-RepositoryGit $root @('rev-parse', '--symbolic-full-name', '@{upstream}')
    $upstream = if ($upstreamResult.ExitCode -eq 0) { $upstreamResult.Output.Trim() } else { $null }
    $ahead = $null
    $behind = $null
    if ($head -and $upstream) {
        $counts = Invoke-RepositoryGit $root @('rev-list', '--left-right', '--count', 'HEAD...@{upstream}')
        if ($counts.ExitCode -eq 0 -and $counts.Output.Trim() -match '^(\d+)\s+(\d+)$') {
            $ahead = [long]$Matches[1]
            $behind = [long]$Matches[2]
        } else {
            $findings.Add([pscustomobject]@{ Code = 'TRACKING_UNKNOWN'; Path = ''; Message = 'Contagem local de commits indisponível.' })
        }
    }
    $index = Invoke-RepositoryGit $root @('ls-files', '--stage', '-z')
    if ($index.ExitCode -ne 0) { throw 'Falha ao consultar arquivos rastreados; diagnóstico incompleto.' }
    $entries = @(
        foreach ($record in ($index.Output -split "`0")) {
            if ($record.Length -eq 0) { continue }
            if ($record -notmatch '(?s)^(\d{6}) ([0-9a-f]+) ([0-3])\t(.+)$') { throw 'Registro de índice inválido.' }
            [pscustomobject]@{ Mode = $Matches[1]; Oid = $Matches[2]; Stage = $Matches[3]; Path = $Matches[4] }
        }
    )
    $sizes = @{}
    $oids = @($entries | Where-Object Mode -ne '160000' | Select-Object -ExpandProperty Oid -Unique)
    if ($oids.Count -gt 0) {
        $objects = Invoke-RepositoryGit $root @('cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)') (($oids -join "`n") + "`n")
        if ($objects.ExitCode -ne 0) { throw 'Falha na leitura local dos tamanhos dos objetos.' }
        foreach ($line in ($objects.Output -split "`n")) {
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            if ($line.Trim() -notmatch '^([0-9a-f]+) blob (\d+)$') { throw 'Objeto local ausente ou inválido; nenhuma busca de rede foi feita.' }
            $sizes[$Matches[1]] = [long]$Matches[2]
        }
        foreach ($oid in $oids) { if (-not $sizes.ContainsKey($oid)) { throw 'Tamanho de objeto local indisponível.' } }
    }
    $trackedPaths = @($entries.Path | Sort-Object -Unique)
    $untrackedResult = Invoke-RepositoryGit $root @('ls-files', '--others', '--exclude-standard', '-z')
    if ($untrackedResult.ExitCode -ne 0) { throw 'Falha ao consultar arquivos não rastreados.' }
    $untracked = @($untrackedResult.Output -split "`0" | Where-Object { $_.Length -gt 0 })
    foreach ($path in @($trackedPaths + $untracked | Sort-Object -Unique)) {
        $tracked = $path -cin $trackedPaths
        $indexBytes = 0L
        $pathEntries = @($entries | Where-Object Path -CEQ $path)
        foreach ($entry in $pathEntries) {
            if ($entry.Mode -eq '160000') {
                $findings.Add([pscustomobject]@{ Code = 'SUBMODULE'; Path = $path; Message = 'Conteúdo de submódulo não auditado por este script.' })
            } else { $indexBytes = [Math]::Max($indexBytes, $sizes[$entry.Oid]) }
            if ($entry.Stage -ne '0') {
                $findings.Add([pscustomobject]@{ Code = 'INDEX_CONFLICT'; Path = $path; Message = 'Conflito no índice; resolução exclusiva do usuário.' })
            }
        }
        $workingBytes = 0L
        $fullPath = Join-Path $root $path
        if (Test-Path -LiteralPath $fullPath) {
            $item = Get-Item -LiteralPath $fullPath -Force -ErrorAction Stop
            if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                $findings.Add([pscustomobject]@{ Code = 'LINK_NOT_SCANNED'; Path = $path; Message = 'Destino de link não percorrido; somente objeto do índice auditado.' })
            } elseif (-not $item.PSIsContainer) { $workingBytes = $item.Length }
        }
        foreach ($finding in @(Get-FilePolicyFindings $path $indexBytes $workingBytes $tracked)) { $findings.Add($finding) }
    }
    foreach ($required in @('AGENTS.md', 'Cargo.lock', '.gitignore', 'docs/internal/memory/')) {
        $type = if ($required.EndsWith('/')) { 'Container' } else { 'Leaf' }
        if (-not (Test-Path -LiteralPath (Join-Path $root $required) -PathType $type)) {
            $findings.Add([pscustomobject]@{ Code = 'REQUIRED_MISSING'; Path = $required; Message = 'Arquivo ou diretório obrigatório ausente.' })
        }
    }
    if (Test-Path -LiteralPath (Join-Path $root 'AGENTS.md') -PathType Leaf) {
        foreach ($problem in @(Get-AgentsStructureFindings (Get-Content -LiteralPath (Join-Path $root 'AGENTS.md') -Raw -ErrorAction Stop))) {
            $findings.Add([pscustomobject]@{ Code = 'AGENTS_STRUCTURE'; Path = 'AGENTS.md'; Message = $problem })
        }
    }
    # Fluxo local do autor: conferido só quando o arquivo existe (não é versionado).
    if (Test-Path -LiteralPath (Join-Path $root 'AGENTS.local.md') -PathType Leaf) {
        $local = Get-Content -LiteralPath (Join-Path $root 'AGENTS.local.md') -Raw -ErrorAction Stop
        foreach ($problem in @(Get-AgentsStructureFindings $local $script:LocalAgentsHeadings)) {
            $findings.Add([pscustomobject]@{ Code = 'AGENTS_STRUCTURE'; Path = 'AGENTS.local.md'; Message = $problem })
        }
    }
    # Provas de comportamento das regras, sem criar arquivos nem alterar o índice.
    $ignoredProbes = @('target/check.bin', 'crates/check/target/check.bin', '.local/check.tmp',
        '.vs/check', 'nested/.idea/check', 'check.pdb', 'check.ilk', 'check.obj', 'check.o', 'check.dmp',
        'check.log', 'check.tmp', 'check.bak', 'check.swp', '.env', '.env.production', 'nested/.env',
        'check.pem', 'check.key', 'backups/check.zip', 'check.vmdk', 'check.vdi', 'check.vhd',
        'check.vhdx', 'check.vmem', 'check.vmss', 'check.qcow2', 'check.iso', '__pycache__/check.pyc')
    $requiredProbes = @('Cargo.toml', 'Cargo.lock', 'AGENTS.md', 'README.md', '.env.example',
        'nested/.env.example', 'docs/internal/spec/check.md', 'docs/internal/memory/check.md', 'docs/internal/reports/check.md',
        'crates/check/src/lib.rs', 'tests/check.rs', 'scripts/check.ps1', '.cargo/config.toml',
        'assets/check.exe', 'assets/check.dll', 'assets/check.svg', 'assets/check.macro')
    $probes = @($ignoredProbes + $requiredProbes + $trackedPaths | Sort-Object -Unique)
    $ignore = Invoke-RepositoryGit $root @('check-ignore', '--no-index', '-z', '--stdin') (($probes -join "`0") + "`0")
    if ($ignore.ExitCode -notin @(0, 1)) { throw 'Não foi possível verificar regras efetivas de ignore.' }
    $ignored = @($ignore.Output -split "`0" | Where-Object { $_.Length -gt 0 })
    foreach ($probe in $ignoredProbes) {
        if ($probe -cnotin $ignored) {
            $findings.Add([pscustomobject]@{ Code = 'IGNORE_MISSING'; Path = $probe; Message = 'Regra de ignore não protege este exemplo.' })
        }
    }
    foreach ($probe in @($requiredProbes + $trackedPaths | Sort-Object -Unique)) {
        if ($probe -cin $ignored) {
            $findings.Add([pscustomobject]@{ Code = 'IGNORE_REQUIRED'; Path = $probe; Message = 'Regra de ignore atinge arquivo necessário ou já rastreado; revisar.' })
        }
    }
    $storage = Invoke-RepositoryGit $root @('count-objects', '-v')
    if ($storage.ExitCode -ne 0) { throw 'Tamanho do armazenamento de objetos indisponível.' }
    $objectBytes = 0L
    foreach ($line in ($storage.Output -split "`n")) {
        if ($line -match '^(size|size-pack|size-garbage): (\d+)') { $objectBytes += [long]$Matches[2] * 1024 }
    }
    if ($objectBytes -ge 1000000000) {
        $findings.Add([pscustomobject]@{ Code = 'REPOSITORY_SIZE'; Path = ''; Message = 'Objetos Git atingem 1 GB; revisar armazenamento. 10 GB é referência operacional, não garantia.' })
    }
    [pscustomobject]@{
        Root = $root; Branch = $branch; Head = $head; Upstream = $upstream
        Ahead = $ahead; Behind = $behind; RemoteState = 'ESTADO REMOTO NÃO VERIFICADO'
        TrackedCount = $trackedPaths.Count; UntrackedCount = $untracked.Count
        ObjectBytes = $objectBytes; Findings = $findings.ToArray()
    }
}

function Write-RepositoryDiagnostics {
    param($Report)
    Write-Output 'CAD-CRAFT — diagnóstico local do repositório'
    Write-Output "Branch: $($Report.Branch)"
    Write-Output "Último commit local: $(if ($Report.Head) { $Report.Head } else { 'indisponível (sem commit ou HEAD inválido)' })"
    Write-Output "Tracking local: $(if ($Report.Upstream) { $Report.Upstream } else { 'indisponível' })"
    if ($null -ne $Report.Ahead) {
        Write-Output "Commits à frente / pendentes de push segundo tracking LOCAL: $($Report.Ahead)"
        Write-Output "Commits atrás segundo tracking LOCAL: $($Report.Behind)"
    } else { Write-Output 'Commits pendentes de push: indeterminados pelos dados locais.' }
    Write-Output $Report.RemoteState
    Write-Output 'Nenhuma consulta de rede. Tracking pode estar desatualizado.'
    Write-Output "Arquivos rastreados: $($Report.TrackedCount); novos não ignorados: $($Report.UntrackedCount)."
    Write-Output "Armazenamento local de objetos: $($Report.ObjectBytes) bytes (count-objects; não inclui todos os metadados .git)."
    Write-Output 'Tamanhos: maior entre índice e arquivo de trabalho; 50/100 MB decimais.'
    foreach ($finding in $Report.Findings) {
        $path = if ($finding.Path) { ConvertTo-Json -InputObject $finding.Path -Compress } else { 'repositório' }
        Write-Output "[$($finding.Code)] $path — $($finding.Message)"
    }
    if ($Report.Findings.Count -eq 0) { Write-Output 'Verificações locais aprovadas: nenhum achado nas regras auditadas.' }
    Write-Output 'Sensíveis: triagem por nome de rastreados/novos não ignorados; sem leitura de conteúdo. Ignorados e histórico completo não são varridos.'
    Write-Output 'AGENTS: conferência estrutural de seções, não validação semântica integral.'
    Write-Output 'Nenhum arquivo foi corrigido ou excluído. Commit, push e backup dependem do usuário.'
}

if ($MyInvocation.InvocationName -ne '.') {
    $ErrorActionPreference = 'Stop'
    try {
        $report = Get-RepositoryDiagnostics -RepositoryPath $RepositoryPath
        Write-RepositoryDiagnostics $report
        if ($report.Findings.Count -gt 0) { exit 1 }
        exit 0
    } catch {
        Write-Output 'ESTADO REMOTO NÃO VERIFICADO'
        Write-Output 'ERRO: diagnóstico local incompleto. Verifique Git, caminho, permissões e objetos locais; nenhuma correção automática foi feita.'
        exit 2
    }
}
