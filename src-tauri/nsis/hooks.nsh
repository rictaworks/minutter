; minutter NSIS installer hooks
; インストール後に Vosk 日本語モデルをダウンロード・展開する

!define VOSK_MODEL_URL "https://github.com/rictaworks/minutter/releases/download/vosk-model/vosk-model-ja.zip"

!macro NSIS_HOOK_POSTINSTALL
  ; 既にモデルが存在する場合はスキップ
  ${If} ${FileExists} "$INSTDIR\models\vosk-model-ja\am\final.mdl"
    DetailPrint "Vosk モデルは既にインストール済みです。スキップします。"
    Goto vosk_model_done
  ${EndIf}

  DetailPrint "Vosk 音声認識モデルをダウンロードしています..."
  DetailPrint "（約 1GB、ネットワーク速度により数分〜十数分かかります）"

  CreateDirectory "$INSTDIR\models"

  ; ダウンロード用 PowerShell スクリプトを一時ファイルに書き出す
  FileOpen $R6 "$TEMP\minutter_dl.ps1" w
  FileWrite $R6 "Start-BitsTransfer -Source '${VOSK_MODEL_URL}' -Destination '$TEMP\vosk-model-ja.zip'"
  FileClose $R6

  ExecWait '"powershell.exe" -NonInteractive -NoProfile -ExecutionPolicy Bypass -File "$TEMP\minutter_dl.ps1"' $R9
  Delete "$TEMP\minutter_dl.ps1"

  ${If} $R9 != 0
    ; BITS 失敗時は Invoke-WebRequest にフォールバック
    DetailPrint "BITS ダウンロード失敗（コード: $R9）。再試行中..."
    FileOpen $R6 "$TEMP\minutter_dl2.ps1" w
    FileWrite $R6 "$$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -Uri '${VOSK_MODEL_URL}' -OutFile '$TEMP\vosk-model-ja.zip' -TimeoutSec 3600"
    FileClose $R6
    ExecWait '"powershell.exe" -NonInteractive -NoProfile -ExecutionPolicy Bypass -File "$TEMP\minutter_dl2.ps1"' $R9
    Delete "$TEMP\minutter_dl2.ps1"
  ${EndIf}

  ${If} $R9 != 0
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "Vosk モデルのダウンロードに失敗しました。$\r$\nインターネット接続を確認してから再インストールしてください。$\r$\nコード: $R9"
    Goto vosk_model_done
  ${EndIf}

  DetailPrint "ダウンロード完了。展開しています（しばらくお待ちください）..."

  ; 展開用 PowerShell スクリプトを一時ファイルに書き出す
  FileOpen $R6 "$TEMP\minutter_extract.ps1" w
  FileWrite $R6 "Expand-Archive -LiteralPath '$TEMP\vosk-model-ja.zip' -DestinationPath '$INSTDIR\models' -Force"
  FileClose $R6

  ExecWait '"powershell.exe" -NonInteractive -NoProfile -ExecutionPolicy Bypass -File "$TEMP\minutter_extract.ps1"' $R9
  Delete "$TEMP\minutter_extract.ps1"

  ${If} $R9 != 0
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "モデルの展開に失敗しました（コード: $R9）。$\r$\n手動で再インストールしてください。"
  ${Else}
    DetailPrint "Vosk モデルのインストールが完了しました。"
    Delete "$TEMP\vosk-model-ja.zip"
  ${EndIf}

  vosk_model_done:
!macroend

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
