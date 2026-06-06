; minutter NSIS installer hooks
; インストール後に Vosk 日本語モデルをダウンロード・展開する

!define VOSK_MODEL_URL "https://github.com/rictaworks/minutter/releases/download/vosk-model/vosk-model-ja.zip"

!macro NSIS_HOOK_POSTINSTALL
  ; --- Vosk モデルのダウンロードと展開 ---

  ; 既にモデルが存在する場合はスキップ
  ${If} ${FileExists} "$INSTDIR\models\vosk-model-ja\am\final.mdl"
    DetailPrint "Vosk モデルは既にインストール済みです。スキップします。"
    Goto vosk_model_done
  ${EndIf}

  DetailPrint "Vosk 音声認識モデルをダウンロードしています..."
  DetailPrint "（約 500MB、ネットワーク速度により数分かかります）"

  ; モデルの保存先ディレクトリを作成
  CreateDirectory "$INSTDIR\models"

  ; ダウンロード先パス
  StrCpy $R8 "$TEMP\vosk-model-ja.zip"

  ; ダウンロード実行
  NSISdl::download "${VOSK_MODEL_URL}" "$R8"
  Pop $R9
  ${If} $R9 != "success"
    DetailPrint "ダウンロード失敗: $R9"
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "Vosk モデルのダウンロードに失敗しました。$\r$\n$\r$\nインターネット接続を確認してから再インストールしてください。$\r$\nエラー: $R9"
    Goto vosk_model_done
  ${EndIf}

  DetailPrint "ダウンロード完了。展開しています（しばらくお待ちください）..."

  ; PowerShell で zip 展開
  StrCpy $R7 "$INSTDIR\models"
  ExecWait 'powershell.exe -NonInteractive -NoProfile -Command "Expand-Archive -LiteralPath $\"$R8$\" -DestinationPath $\"$R7$\" -Force"' $R9
  ${If} $R9 != 0
    DetailPrint "展開失敗（コード: $R9）"
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "モデルの展開に失敗しました（コード: $R9）。$\r$\n手動で再インストールしてください。"
  ${Else}
    DetailPrint "Vosk モデルのインストールが完了しました。"
  ${EndIf}

  ; 一時ファイル削除
  Delete "$R8"

  vosk_model_done:
!macroend

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
