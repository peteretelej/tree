@echo off
title Tree CLI

echo.
echo Tree CLI - treecli command
echo --------------------------------
echo An alternative to Windows 'tree' command for visualizing directories in a tree-like format.
echo.

echo USAGE (open any command prompt or terminal):
echo   treecli                    # Show current directory
echo   treecli C:\Projects        # Show specific directory  
echo   treecli -L 2               # Limit depth to 2 levels
echo.

echo COMMON OPTIONS:
echo   -a    Show hidden files
echo   -d    Show directories only
echo   -s    Show file sizes
echo   -C    Colorize output
echo.

echo For all options: treecli --help
echo.

echo Please run the command in a terminal or command prompt: treecli
cmd /k