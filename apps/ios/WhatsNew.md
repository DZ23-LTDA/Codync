<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.21.0

### zh-Hant
- Routine 還沒動手就失敗（例如額度用完）時，會在 1、5、15 分鐘後自動再試；已經動過手或超過時間上限的失敗，通知旁有重試按鈕讓你決定
- Bot 的記憶頁改成唯讀，想改記憶直接跟 bot 說；Dreams 分頁新增的記憶是綠色、封存的是紅色，旁邊的資訊按鈕說明做夢在做什麼

### en-US
- A routine that fails before doing anything (a usage limit, say) tries again on its own after 1, 5 and 15 minutes; one that already acted or hit its time limit gets a retry button on its failure notice, so you decide
- A bot's memory is now read-only: to change it, tell the bot. In the Dreams tab, added memories show in green, archived ones in red, and an info button explains what dreaming does
