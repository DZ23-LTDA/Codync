<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.33.0

### zh-Hant
- 任何 Bot 都能當主助理：長按 Bot 選「設為主助理」，Siri、動作按鈕和沒有指定 Bot 的「Open Live Mode」捷徑就會改找它。
- 主助理可以像其他 Bot 一樣拖曳排序、釘選或移到分組，不需要時也可以直接刪除。
- 有多台電腦時，可以在「設定 > 捷徑」選擇動作按鈕要呼叫哪一台電腦的主助理。
- Bot 選單更簡潔：不再需要手動開新的工作階段，Codync 會在需要時自動接續。
- 選專案資料夾時，返回鍵可以往上走到家目錄以外的資料夾，Windows 電腦還能切換磁碟；連結（symlink）的資料夾也會出現在清單中。

### en-US
- Any bot can be your lead: touch and hold it and choose Make lead agent. Siri, the Action button and Open Live Mode without a bot then call it.
- Drag, pin or move the lead into a section like any bot, or delete it if you don't need one.
- With more than one computer, choose whose lead the Action button calls in Settings > Shortcuts.
- A simpler bot menu: no more New session, since Codync starts a fresh one on its own when a conversation needs it.
- When you choose a project folder, Back climbs above your home folder and, on a Windows computer, to its other drives; linked folders now show up too.
