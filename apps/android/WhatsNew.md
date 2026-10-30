<!--
Google Play release notes for the Android app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per Play language (zh-Hant, published as zh-TW;
en-US), listing what Android users notice, each under 500 characters (Play's limit;
tools/play-submit.py cuts longer notes at the last whole line). tools/play-submit.py reads the
released version's section; a version without one gets the PRs' "What's New: Android" bullets,
else a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.33.2

### zh-Hant
- 電腦的推送通知設定會顯示系統權限狀態；未允許通知時，可直接前往開啟權限。

### en-US
- Computer push settings show when notification permission is missing and help you enable it.

## 2.33.1

### zh-Hant
- 選專案資料夾時，返回鍵可以往上走到家目錄以外的資料夾，Windows 電腦還能切換磁碟；連結（symlink）的資料夾也會出現在清單中。

### en-US
- When you choose a project folder, Back climbs above your home folder and, on a Windows computer, to its other drives; linked folders now show up too.
