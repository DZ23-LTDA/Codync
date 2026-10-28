<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.31.1

### zh-Hant
- 在「擴充套件」裡打開一個 Agent，登入下方會列出它自己的技能、聯結器、規則和外掛。
- Codex 的用量改成讀取目前登入帳號的額度，不會再停在舊的百分比。
- 從手機更新電腦時，電腦重新啟動的期間會顯示「正在更新…」，不會再要你申請訪問。

### en-US
- Open an agent in Extensions. Under Sign in, see the skills, connectors, rules and plugins it already has.
- Codex usage now shows your signed-in account's current allowance instead of an old percentage.
- While a computer restarts for an update you started from your phone, it shows Updating… instead of asking you for access.
