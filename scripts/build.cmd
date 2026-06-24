@echo off
setlocal EnableExtensions
cd /d "%~dp0.."

where python >nul 2>nul
if %errorlevel%==0 (
  set "PYTHON=python"
) else (
  where py >nul 2>nul
  if %errorlevel%==0 (
    set "PYTHON=py -3.12"
  ) else (
    echo ERROR: Python not found. Install Python 3.12 and enable Add to PATH.
    pause
    exit /b 1
  )
)

%PYTHON% scripts\build.py all
if errorlevel 1 goto fail

echo.
echo SUCCESS: dist\mfhq.exe
pause
exit /b 0

:fail
echo.
echo ERROR: build failed.
pause
exit /b 1
