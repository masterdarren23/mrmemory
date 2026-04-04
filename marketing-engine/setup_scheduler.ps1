# Marketing Engine — Windows Task Scheduler Setup
# Run this script as Administrator to create all scheduled tasks

$python = "C:\Users\johnl\AppData\Local\Programs\Python\Python311\python.exe"
$script = "C:\Users\johnl\.openclaw\workspace\marketing-engine\run_agent.py"
$workdir = "C:\Users\johnl\.openclaw\workspace\marketing-engine"

# Helper function
function Create-MarketingTask {
    param($Name, $Agent, $Schedule, $Time, $DaysOfWeek)
    
    $action = New-ScheduledTaskAction -Execute $python -Argument $script" "$Agent -WorkingDirectory $workdir
    
    if ($DaysOfWeek) {
        $trigger = New-ScheduledTaskTrigger -Weekly -DaysOfWeek $DaysOfWeek -At $Time
    } else {
        $trigger = New-ScheduledTaskTrigger -Daily -At $Time
    }
    
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
    
    Register-ScheduledTask -TaskName "MKT_$Name" -Action $action -Trigger $trigger -Settings $settings -Description "MrMemory Marketing: $Name" -Force
    Write-Host "Created: MKT_$Name ($Schedule at $Time)" -ForegroundColor Green
}

Write-Host "Setting up Marketing Engine scheduled tasks..." -ForegroundColor Cyan
Write-Host ""

# DAILY agents
Create-MarketingTask -Name "SocialListening"  -Agent "social_listening"  -Schedule "Daily"  -Time "08:00"
Create-MarketingTask -Name "SocialMedia"      -Agent "social_media"      -Schedule "Daily"  -Time "09:00"
Create-MarketingTask -Name "MediaOutreach"    -Agent "media_outreach"    -Schedule "Daily"  -Time "10:00"
Create-MarketingTask -Name "CommunityReply"   -Agent "community_reply"   -Schedule "Daily"  -Time "11:00"

# WEEKLY agents  
Create-MarketingTask -Name "GrowthMetrics"    -Agent "growth_metrics"    -Schedule "Weekly" -Time "09:00" -DaysOfWeek "Monday"
Create-MarketingTask -Name "CVR"              -Agent "cvr"               -Schedule "Weekly" -Time "09:00" -DaysOfWeek "Wednesday"
Create-MarketingTask -Name "SEO"              -Agent "seo"               -Schedule "Weekly" -Time "09:00" -DaysOfWeek "Tuesday"
Create-MarketingTask -Name "GEO"              -Agent "geo"               -Schedule "Weekly" -Time "09:00" -DaysOfWeek "Thursday"
Create-MarketingTask -Name "CompetitorWatch"  -Agent "competitor_watch"  -Schedule "Weekly" -Time "09:00" -DaysOfWeek "Wednesday"
Create-MarketingTask -Name "WeeklyStrategy"   -Agent "weekly_strategy"   -Schedule "Weekly" -Time "14:00" -DaysOfWeek "Friday"

Write-Host ""
Write-Host "All tasks created! View them with: Get-ScheduledTask | Where TaskName -like 'MKT_*'" -ForegroundColor Cyan
Write-Host ""
Write-Host "Schedule:" -ForegroundColor Yellow
Write-Host "  DAILY:"
Write-Host "    08:00 - Social Listening (scan communities)"
Write-Host "    09:00 - Social Media (create posts)"
Write-Host "    10:00 - Media Outreach (find targets, draft pitches)"
Write-Host "    11:00 - Community Reply (draft helpful replies)"
Write-Host "  WEEKLY:"
Write-Host "    Mon 09:00 - Growth Metrics"
Write-Host "    Tue 09:00 - SEO"
Write-Host "    Wed 09:00 - CVR + Competitor Watch"
Write-Host "    Thu 09:00 - GEO"
Write-Host "    Fri 14:00 - Weekly Strategy (synthesis)"
