@echo off
chcp 65001 >nul
title Desinstalar parche GALAXYZ neo - Español Latino

set "DATA=%LOCALAPPDATA%\fuzz\galaxyz\data"
set "BACKUP=%DATA%\backup_antes_del_parche"

if not exist "%BACKUP%" (
    echo [ERROR] No se encontro la copia de seguridad:
    echo   %BACKUP%
    echo Si no la tienes, borra las carpetas epk del juego y deja que las vuelva a descargar.
    pause
    exit /b 1
)

echo Cierra el juego antes de continuar.
pause

for %%D in ("locale\us\epk" "epk" "root\epk") do (
    if exist "%BACKUP%\%%~D" (
        copy /Y "%BACKUP%\%%~D\*.epk" "%DATA%\%%~D\" >nul
        echo   Restaurado  %%~D
    )
)

echo.
echo Archivos originales restaurados.
pause
