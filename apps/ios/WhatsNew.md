<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.20.3

### zh-Hant
- 更改連線方式（例如不經過 Cloudflare）會立即生效，重新配對同一台電腦也會保留設定
- 移除透過帳號核准的電腦時，會一併撤銷它的存取權；手機離線時也能移除
- 從帳號移除舊的電腦後，它不會再跑回清單；存取請求被撤回時會說明原因
- 設定裡的帳號操作失敗時會顯示錯誤，不再沒有反應
- 新增 Bot 時不再列出已無法使用的舊電腦

### en-US
- Changing how you connect, such as never using Cloudflare, takes effect right away and stays when you pair the same computer again
- Removing a computer approved through your account also revokes its access, even while your iPhone is offline
- A computer removed from your account no longer reappears in the list, and a withdrawn access request says why
- Account actions in Settings show an error when they fail instead of doing nothing
- Computers that can no longer be used are left out when you create a bot
