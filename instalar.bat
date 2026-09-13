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

echo El parche se instalara en:
echo   %DATA%
echo.
echo Cierra el juego antes de continuar.
pause

rem Copia de seguridad (solo la primera vez y solo si ya habia .epk sueltos,
rem por ejemplo de otro mod). Los archivos originales del juego estan dentro
rem de su OBB y nunca se modifican.
if not exist "%BACKUP%" (
    for %%D in ("locale\us\epk" "epk" "root\epk") do (
        if exist "%DATA%\%%~D\*.epk" (
            mkdir "%BACKUP%\%%~D" 2>nul
            copy /Y "%DATA%\%%~D\*.epk" "%BACKUP%\%%~D\" >nul
            echo Copia de seguridad de %%~D guardada.
        )
    )
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
echo.
echo Si el juego se cierra mientras carga la pantalla de inicio, es normal:
echo sigue abriendolo hasta que entre.
pause
