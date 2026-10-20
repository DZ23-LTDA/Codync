<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.20.4

### zh-Hant
- 拖曳電腦標題時，它的 Bot 會跟著一起移動：拖曳時各電腦的 Bot 先收起，放開後在各自的電腦底下展開
- Bot 只會在自己的電腦裡排序；拖到另一台電腦的 Bot 之間時，會排到自己電腦的最前面或最後面

### en-US
- Dragging a computer's heading moves its bots with it: every computer's bots fold away while you drag and unfold under their own computer when you let go
- Bots reorder only within their own computer; dropping one among another computer's bots puts it first or last on its own
