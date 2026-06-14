@echo off
REM Wrapper qui lance uninstall.ps1 sans buter sur la ExecutionPolicy
REM par defaut de Windows (Restricted). Double-clique uninstall.bat.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1"
