<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.29.0

### zh-Hant
- Agent、模型、推理強度與 Fast mode 整合成可收合的精簡設定；每個 Bot 可獨立保存推理強度，Default 也會顯示預設值。
- 評分邀請只會在你使用一段時間、看完 Bot 的成果並回到列表後出現，不會在開啟 App 時打擾你。也可以隨時從設定前往 App Store 評分。

### en-US
- Agent, model, reasoning effort and Fast mode now share compact, expandable settings. Each bot keeps its own effort preference, with the default level shown.
- Rating invitations wait until you have used Codync for a while, viewed a bot's result, and returned to the list. You can also rate Codync anytime from Settings.
