$headers = @{ Authorization = "Bearer amr_sk_test_live_key_001" }

Write-Host "=== Creating memory ==="
$body = '{"content":"User loves Tesla Model 3 with autopilot and wants one under 40k","tags":["tesla","budget"],"namespace":"cars"}'
$result = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories" -Method POST -Headers $headers -ContentType "application/json" -Body $body
Write-Host "Created: $($result.id)"

Write-Host "`n=== Waiting 4s for embedding ==="
Start-Sleep -Seconds 4

Write-Host "=== Semantic recall ==="
$recall = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories/recall?query=electric+car+preferences&limit=5&threshold=0.3" -Method GET -Headers $headers
Write-Host "Results: $($recall.count)"
$recall | ConvertTo-Json -Depth 5
