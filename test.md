cd C:\Users\RKerrigan\Projects\pp2p
& .\.venv\Scripts\Activate.ps1
Get-Process tor -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-Item .\human_test -Recurse -Force -ErrorAction SilentlyContinue
.\.venv\Scripts\python.exe pp2p.py init --state-dir .\human_test\peerA
.\.venv\Scripts\python.exe pp2p.py init --state-dir .\human_test\peerB
.\.venv\Scripts\python.exe pp2p.py invite --state-dir .\human_test\peerA --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe | Out-File -Encoding utf8 .\human_test\peerA_invite.json
.\.venv\Scripts\python.exe pp2p.py invite --state-dir .\human_test\peerB --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe | Out-File -Encoding utf8 .\human_test\peerB_invite.json

peer a
.\.venv\Scripts\python.exe pp2p.py run --state-dir .\human_test\peerA --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe

peer a at pp2p
/add-file .\human_test\peerB_invite.json
/peers


peer b
.\.venv\Scripts\python.exe pp2p.py run --state-dir .\human_test\peerB --mode onion --tor-bin .\tor_win_min_src\src\app\tor.exe

peer b at pp2p
/add-file .\human_test\peerA_invite.json
/peers

/send <other_peer_id> hi
