!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$SYSDIR\sc.exe" stop DriverUpdaterService'
  Sleep 1000
  ExecWait '"$SYSDIR\sc.exe" delete DriverUpdaterService'
  Sleep 500
  Delete "$INSTDIR\driver-updater-service.exe"
  Delete "$COMMONPROGRAMDATA\Driver Updater\service.token"
  RMDir "$COMMONPROGRAMDATA\Driver Updater"
!macroend
