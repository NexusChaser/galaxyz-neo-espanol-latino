@echo off
chcp 65001 >nul
title Parche GALAXYZ neo - Español Latino

set "SRC=%~dp0epk"
set "DATA=%LOCALAPPDATA%\fuzz\galaxyz\data"
set "BACKUP=%DATA%\backup_antes_del_parche"

echo ================================================
echo   Parche de traduccion al Español Latino
echo   GALAXYZ neo
echo ================================================
echo.

if not exist "%SRC%\scene_data.epk" (
    echo [ERROR] No se encontro la carpeta "epk" junto a este instalador.
    echo Descomprime el ZIP completo y vuelve a ejecutar instalar.bat
    pause
    exit /b 1
)

if not exist "%DATA%" (
    echo [ERROR] No se encontro la carpeta del juego:
    echo   %DATA%
    echo Abre el juego al menos una vez, deja que descargue los datos,
    echo cierralo y vuelve a ejecutar este instalador.
    pause
    exit /b 1
)

echo Cierra el juego antes de continuar.
pause

rem Copia de seguridad (solo la primera vez, para no pisar los originales)
if not exist "%BACKUP%" (
    echo Creando copia de seguridad en:
    echo   %BACKUP%
    for %%D in ("locale\us\epk" "epk" "root\epk") do (
        if exist "%DATA%\%%~D" (
            mkdir "%BACKUP%\%%~D" 2>nul
            copy /Y "%DATA%\%%~D\*.epk" "%BACKUP%\%%~D\" >nul
        )
    )
) else (
    echo Ya existe una copia de seguridad, no se sobrescribe.
)

echo.
echo Instalando traduccion...
for %%D in ("locale\us\epk" "epk" "root\epk") do (
    mkdir "%DATA%\%%~D" 2>nul
    copy /Y "%SRC%\*.epk" "%DATA%\%%~D\" >nul
    if errorlevel 1 (
        echo [ERROR] No se pudo copiar a %DATA%\%%~D
        pause
        exit /b 1
    )
    echo   OK  %%~D
)

echo.
echo ¡Listo! Ya puedes abrir el juego en español.
pause
