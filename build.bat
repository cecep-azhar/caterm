@echo off
set VERSION=%1
if "%VERSION%"=="" set VERSION=0.1

echo ==============================================
echo  Building CATerm v%VERSION% for Windows
echo ==============================================

wails build -platform windows/amd64 -o caterm-%VERSION%.exe

if exist "build\bin\caterm-%VERSION%.exe" (
    echo.
    echo [SUCCESS] Build completed!
    echo Output: build\bin\caterm-%VERSION%.exe
) else (
    echo.
    echo [ERROR] Build failed. Please check Wails & Go setup.
)
