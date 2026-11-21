!macro customUnInstall
  IfSilent +4 0
  MessageBox MB_YESNO|MB_ICONQUESTION "¿Deseas CONSERVAR tus instancias, mundos guardados, skins y base de datos en ~/.omnicraft_studio para futuras instalaciones?$\r$\n$\r$\n• Sí = Conservar todos mis archivos y configuraciones (Recomendado)$\r$\n• No = Eliminar TODO lo relacionado al gestor (~/.omnicraft_studio)" IDYES keep_omnicraft_data
  RMDir /r "$PROFILE\.omnicraft_studio"
  keep_omnicraft_data:
!macroend
