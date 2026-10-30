<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.33.2

### zh-Hant
- 語音輸入尚未設定 API key 時，會直接開啟語音設定。

### en-US
- Dictation opens voice settings when your provider API key is missing.

## 2.33.1

### zh-Hant
- 選專案資料夾時，返回鍵可以往上走到家目錄以外的資料夾，Windows 電腦還能切換磁碟；連結（symlink）的資料夾也會出現在清單中。

### en-US
- When you choose a project folder, Back climbs above your home folder and, on a Windows computer, to its other drives; linked folders now show up too.
