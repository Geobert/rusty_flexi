@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2017\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cargo build --release
if %ERRORLEVEL% EQU 0 "C:\Program Files (x86)\Resource Hacker\ResourceHacker.exe" -open .\target\release\rusty_flexi.exe -save .\rusty_flexi.exe -action addskip -res .\res\clock_red.ico -mask ICONGROUP,MAINICON, -log CONSOLE
