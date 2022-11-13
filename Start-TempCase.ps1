$in = ".\in.txt"
$env:DURATION_MUL = "0.5"
Get-Content $in | .\tester.exe cargo run --release > out.txt
