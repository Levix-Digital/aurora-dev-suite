; Installer hooks: put the installation folder, which holds aurora.exe and
; dcp-integrations.exe, on the user's PATH, and take it off on uninstall.
; HKCU only (installMode currentUser); no plugins.

!define /math AURORA_PATH_LIMIT ${NSIS_MAX_STRLEN} - 2

!macro NSIS_HOOK_POSTINSTALL
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  ReadRegStr $0 HKCU "Environment" "Path"
  ; A value near the NSIS string limit may have been cut: leave PATH alone.
  StrLen $3 "$0;$INSTDIR"
  IntCmp $3 ${AURORA_PATH_LIMIT} aurora_path_long 0 aurora_path_long
  StrCpy $1 ";$0;"
  StrCpy $2 ";$INSTDIR;"
  StrLen $3 $2
  StrCpy $4 0
  aurora_path_search:
    StrCpy $5 $1 $3 $4
    StrCmp $5 "" aurora_path_add
    StrCmp $5 $2 aurora_path_done
    IntOp $4 $4 + 1
    Goto aurora_path_search
  aurora_path_add:
    StrCmp $0 "" 0 aurora_path_append
    StrCpy $0 "$INSTDIR"
    Goto aurora_path_write
  aurora_path_append:
    StrCpy $5 $0 1 -1
    StrCmp $5 ";" 0 aurora_path_separator
    StrCpy $0 "$0$INSTDIR"
    Goto aurora_path_write
  aurora_path_separator:
    StrCpy $0 "$0;$INSTDIR"
  aurora_path_write:
    WriteRegExpandStr HKCU "Environment" "Path" $0
    SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000
    Goto aurora_path_done
  aurora_path_long:
    DetailPrint "PATH is too long to change safely; add $INSTDIR to it by hand."
  aurora_path_done:
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  ReadRegStr $0 HKCU "Environment" "Path"
  StrLen $3 "$0;"
  IntCmp $3 ${AURORA_PATH_LIMIT} aurora_unpath_done 0 aurora_unpath_done
  StrCpy $1 ";$0;"
  StrCpy $2 ";$INSTDIR;"
  StrLen $3 $2
  StrCpy $4 0
  aurora_unpath_search:
    StrCpy $5 $1 $3 $4
    StrCmp $5 "" aurora_unpath_done
    StrCmp $5 $2 aurora_unpath_found
    IntOp $4 $4 + 1
    Goto aurora_unpath_search
  aurora_unpath_found:
    ; keep what comes before ";<dir>" and after it, from its closing ";"
    StrCpy $5 $1 $4
    IntOp $6 $4 + $3
    IntOp $6 $6 - 1
    StrCpy $6 $1 "" $6
    StrCpy $1 "$5$6"
    ; drop the ";" added at both ends
    StrCpy $1 $1 "" 1
    StrCpy $1 $1 -1
    WriteRegExpandStr HKCU "Environment" "Path" $1
    SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000
  aurora_unpath_done:
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
!macroend
