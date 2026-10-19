<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.20.1

### zh-Hant

- 登入後不會再自動向帳號裡的電腦要求存取，也不會直接跳出驗證碼；在「電腦」裡點「Ask for access」才會開始。

### en-US

- Signing in no longer asks your account's computers for access on its own or jumps straight to a code; it starts when you tap Ask for access in Computers.
