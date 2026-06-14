@echo off
REM Wrapper qui lance install.ps1 sans buter sur la ExecutionPolicy
REM par defaut de Windows (Restricted). Double-clique install.bat,
REM rien d'autre a faire.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
