Add-Type -AssemblyName System.IO.Compression
$dir = "E:\PYTHON\CoreSwap\runtime\1.20.1\java\run\logs"
$files = Get-ChildItem "$dir\*.log.gz" | Sort-Object LastWriteTime -Descending | Select-Object -First 8
foreach ($f in $files) {
    $fs = [System.IO.File]::OpenRead($f.FullName)
    $gz = New-Object System.IO.Compression.GZipStream($fs, [System.IO.Compression.CompressionMode]::Decompress)
    $sr = New-Object System.IO.StreamReader($gz)
    $content = $sr.ReadToEnd()
    $sr.Close(); $fs.Close()
    $lines = $content -split "`n" | Select-String -SimpleMatch "[Mixin]"
    $neth = $lines | Select-String -SimpleMatch "(nether)"
    $plain = $lines | Where-Object { $_ -notmatch "nether" }
    Write-Output ("== {0}  total={1} nether={2} overworld-shape={3}" -f $f.Name, $lines.Count, $neth.Count, $plain.Count)
    $lines | Select-Object -First 5 | ForEach-Object { Write-Output ("   " + $_.ToString().Trim()) }
}
