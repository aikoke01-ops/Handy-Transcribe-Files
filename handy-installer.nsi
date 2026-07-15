; Handy Installer Script - Improved
Name "Handy 0.9.2"
OutFile "C:\Users\Koke2025\Handy\Handy_0.9.2_Installer.exe"
InstallDir "$PROGRAMFILES\Handy"

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  
  ; Copy main executable
  File "C:\Users\Koke2025\Handy\src-tauri\target\release\handy.exe"
  
  ; Copy ALL DLL files
  File /r "C:\Users\Koke2025\Handy\src-tauri\target\release\*.dll"
  
  ; Create shortcuts
  CreateDirectory "$SMPROGRAMS\Handy"
  CreateShortcut "$SMPROGRAMS\Handy\Handy.lnk" "$INSTDIR\handy.exe"
  CreateShortcut "$DESKTOP\Handy.lnk" "$INSTDIR\handy.exe"
  CreateShortcut "$SMPROGRAMS\Handy\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  
  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\handy.exe"
  Delete "$INSTDIR\*.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Handy\*.*"
  RMDir "$SMPROGRAMS\Handy"
  Delete "$DESKTOP\Handy.lnk"
SectionEnd