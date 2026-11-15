@echo off
if exist "%~dp0dist\iskandil\iskandil.exe" (
  start "" "%~dp0dist\iskandil\iskandil.exe" %*
  exit /b
)
if exist "%~dp0target\release\iskandil.exe" (
  start "" "%~dp0target\release\iskandil.exe" %*
  exit /b
)
if exist "%~dp0target\debug\iskandil.exe" (
  start "" "%~dp0target\debug\iskandil.exe" %*
  exit /b
)
echo Iskandil.exe bulunamadi. Once scripts\build-release.ps1 calistirin.
