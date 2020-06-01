!include "WordFunc.nsh"

Name "Radical Native"
Outfile ..\..\target\release\radical-native-installer.exe
Unicode true
InstallDir $PROGRAMFILES64\radical-native
 
Section
ExecWait "TaskKill /IM radical-native.exe /F"
 
SetOutPath $INSTDIR
CreateDirectory $INSTDIR
 
File ..\..\target\release\radical-native.exe

${WordReplace} $INSTDIR "\" "\\" "+" $R0
FileOpen $0 "$INSTDIR\radical.native.json" w
FileWrite $0 '{$\r$\n\
    "name": "radical.native",$\r$\n\
    "description": "Radical Native",$\r$\n\
    "path": "$R0\\radical-native.exe",$\r$\n\
    "type": "stdio",$\r$\n\
    "allowed_extensions": [ "@radical-native", "@riot-webext" ]$\r$\n\
}'
FileClose $0

SetRegView 64
WriteRegStr HKLM "Software\Mozilla\NativeMessagingHosts\radical.native" "" "$INSTDIR\radical.native.json"

WriteUninstaller $INSTDIR\uninstaller.exe
SectionEnd
 
Section "Uninstall"
ExecWait "TaskKill /IM radical-native.exe /F"
Delete $INSTDIR\uninstaller.exe
Delete $INSTDIR\radical-native.exe
Delete $INSTDIR\radical.native.json
DeleteRegKey HKLM "Software\Mozilla\NativeMessagingHosts\radical.native"
RMDir $INSTDIR
SectionEnd