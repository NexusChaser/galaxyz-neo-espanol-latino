@echo off
chcp 65001 >nul
title Desinstalar parche GALAXYZ neo - Español Latino

set "SRC=%~dp0epk"
set "DATA=%LOCALAPPDATA%\fuzz\galaxyz\data"
set "BACKUP=%DATA%\backup_antes_del_parche"

if not exist "%SRC%\scene_data.epk" (
    echo [ERROR] No se encontro la carpeta "epk" junto a este desinstalador.
    echo Hace falta para saber que archivos son del parche.
    pause
    exit /b 1
)

echo Se quitaran los archivos del parche de:
echo   %DATA%
echo El juego volvera a usar sus textos originales.
echo.
echo Cierra el juego antes de continuar.
pause

rem Borrar solo los archivos del parche
for %%D in ("locale\us\epk" "epk" "root\epk") do (
    if exist "%DATA%\%%~D" (
        for %%F in ("%SRC%\*.epk") do del /Q "%DATA%\%%~D\%%~nxF" 2>nul
        echo   Quitado  %%~D
    )
)

rem Devolver los archivos que habia antes del parche (si los habia)
if exist "%BACKUP%" (
    for %%D in ("locale\us\epk" "epk" "root\epk") do (
        if exist "%BACKUP%\%%~D\*.epk" (
            copy /Y "%BACKUP%\%%~D\*.epk" "%DATA%\%%~D\" >nul
            echo   Restaurado  %%~D
        )
    )
)

rem Quitar las carpetas que hayan quedado vacias (rd no borra carpetas con contenido)
for %%D in ("locale\us\epk" "locale\us" "locale" "epk" "root\epk" "root") do rd "%DATA%\%%~D" 2>nul

echo.
echo Parche desinstalado.
pause
