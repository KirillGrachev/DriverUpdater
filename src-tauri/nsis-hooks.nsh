!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$SYSDIR\sc.exe" stop DriverUpdaterService'
  Sleep 1000
  ExecWait '"$SYSDIR\sc.exe" delete DriverUpdaterService'
  Sleep 500
  Delete "$PROGRAMFILES64\Driver Updater\driver-updater-service.exe"
  Delete "$PROGRAMFILES\Driver Updater\driver-updater-service.exe"
  RMDir "$PROGRAMFILES64\Driver Updater"
  RMDir "$PROGRAMFILES\Driver Updater"
  Delete "$COMMONPROGRAMDATA\Driver Updater\service.token"
  RMDir "$COMMONPROGRAMDATA\Driver Updater"
!macroend
