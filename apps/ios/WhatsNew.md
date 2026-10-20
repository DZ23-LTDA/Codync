<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.20.2

### zh-Hant

- 新增 Bot 改成先設定外觀：挑形狀和顏色、取個名字（也可以不取），按一下就建立並開啟聊天。其他設定之後再到 Bot 的設定裡調整。
- 建立群組聊天也用同樣的方式：選好成員、取個名字（可留空），按一下就開啟群組。
- 群組聊天裡輸入「/」也能用指令了：一次幫所有成員開關 Fast mode，或叫出任一成員的技能，每個有這項技能的 Bot 都會照著做。
- 登入後不會再自動向帳號裡的電腦要求存取，也不會直接跳出驗證碼；在「電腦」裡點「Ask for access」才會開始。
- 長按任何訊息（你的或 Bot 的）選「回覆」，就能直接在聊天裡引用那則訊息回覆。討論串功能拿掉了，原本討論串裡的訊息都搬回主聊天，照時間排好。
- 在 Bot 設定裡打開「Manage the team」，這個 Bot 就能幫你建立新 Bot，也能調整自己和其他 Bot 的指示、模型、頭像和例行工作；它建立的 Bot 同樣能管理團隊。權限設為「Ask me」時，每次變更都會先跳出核准卡片，列出要做的所有內容。由 Bot 建立的 Bot，設定裡會顯示是誰、在哪一天建立的。

### en-US

- Creating a bot starts with how it looks: pick a shape and a color, give it a name if you like, and its chat opens. Everything else stays in the bot's Settings for later.
- A new group chat works the same way: pick its members, name it if you like, and the group opens.
- Typing / in a group chat now lists commands too: turn Fast mode on or off for every bot at once, or pick any member's skill and each bot that has it follows it.
- Signing in no longer asks your account's computers for access on its own or jumps straight to a code; it starts when you tap Ask for access in Computers.
- Long-press any message, yours or a bot's, and tap Reply to quote it right in the chat. Threads are gone: their messages now sit in the main chat, in time order.
- Turn on Manage the team in a bot's settings and it can create bots for you and change its own and other bots' instructions, model, avatar and routines; the bots it creates can manage the team too. With Ask me, every change first shows an approval card listing all of it. A bot created by another bot shows who created it, and when, in its settings.
