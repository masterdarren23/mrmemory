$headers = @{ Authorization = "Bearer amr_sk_test_live_key_001" }

Write-Host "=== Creating memory with embedding ==="
$body = '{"content":"The customer prefers electric vehicles with long range batteries","tags":["ev","preference"],"namespace":"auto"}'
$result = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories" -Method POST -Headers $headers -ContentType "application/json" -Body $body
$result | ConvertTo-Json
Write-Host ""

Write-Host "=== Waiting 3s for embedding generation ==="
Start-Sleep -Seconds 3

Write-Host "=== Semantic recall (should match with REAL score) ==="
$recall = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories/recall?query=What+kind+of+car+does+the+customer+want" -Method GET -Headers $headers
$recall | ConvertTo-Json -Depth 5
Write-Host ""

Write-Host "=== Readiness check ==="
$ready = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/readiness" -Method GET
$ready | ConvertTo-Json -Depth 3
