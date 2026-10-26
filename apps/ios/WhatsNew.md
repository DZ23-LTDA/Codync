<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.29.0

### zh-Hant
- 評分邀請只會在你使用一段時間、看完 Bot 的成果並回到列表後出現，不會在開啟 App 時打擾你。也可以隨時從設定前往 App Store 評分。
- 設定裡每台電腦多了網路按鈕（Beta）：一台電腦可以加入多個網路，這裡列出每個網路裡有哪些電腦、是否在線、各自提供哪些 Bot、誰可以傳訊息給這裡的 Bot，以及雲端助理是否開啟。網路的建立與邀請要在電腦上的 Codync 進行。
- Bot 設定多了「提供給網路」與「雲端助理可聯繫」兩個開關，決定其他電腦上的 Bot 與 ChatGPT、Claude 能不能找到它。

### en-US
- Rating invitations wait until you have used Codync for a while, viewed a bot's result, and returned to the list. You can also rate Codync anytime from Settings.
- A network button (beta) on each computer in Settings: a computer can be in several networks, and it shows each one's computers, whether each answers, the bots it offers, who may message the bots here, and whether cloud assistants are on. You set up and invite computers in Codync on a computer.
- Bot settings have Offered to the network and Reachable by cloud assistants, so you choose whether bots on your other computers, ChatGPT or Claude can find a bot.
