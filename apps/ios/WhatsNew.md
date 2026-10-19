<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.20.0

### zh-Hant

- 設定 Routine 排程更簡單：選擇每小時、每天、平日或每週，再挑時間就好；需要更細的排程時，仍可用「自訂」直接輸入 cron。
- 時區改成可搜尋的清單，打幾個字就能找到，預設是你目前所在的時區。
- 新版本一上架，更新提醒就會出現，不再晚上一天。

### en-US

- Setting a routine's schedule is simpler: pick hourly, daily, weekdays or weekly, then the time. Custom still takes a cron for anything finer.
- Time zones come from a searchable list, found in a few keystrokes, starting from the one you're in.
- The update reminder shows up as soon as a new version is in the App Store, not a day later.
