@echo off
cargo build --release
"C:\Program Files (x86)\Resource Hacker\ResourceHacker.exe" -open .\target\release\rusty_flexi.exe -save .\rusty_flexi.exe -action addskip -res .\res\clock_red.ico -mask ICONGROUP,MAINICON, -log CONSOLE