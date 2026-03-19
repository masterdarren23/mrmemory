$headers = @{ Authorization = "Bearer amr_sk_test_live_key_001" }
$body = '{"content":"Customer wants a weather prediction model","tags":["weather","prediction"],"namespace":"test"}'

Write-Host "=== Creating memory ==="
$result = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories" -Method POST -Headers $headers -ContentType "application/json" -Body $body
$result | ConvertTo-Json

Write-Host "`n=== Listing memories ==="
$list = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories" -Method GET -Headers $headers
$list | ConvertTo-Json -Depth 5

Write-Host "`n=== Recalling memories ==="
$recallBody = '{"query":"What does the customer want?","limit":5,"threshold":0.5}'
$recall = Invoke-RestMethod -Uri "https://amr-memory-api.fly.dev/v1/memories/recall" -Method GET -Headers $headers -Body "query=What+does+the+customer+want"
$recall | ConvertTo-Json -Depth 5
