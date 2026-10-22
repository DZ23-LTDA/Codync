<!--
App Store "What's New" for the iPhone app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per App Store language (zh-Hant, en-US),
listing what iPhone users notice, without emoji (the App Store rejects them; asc-submit.py drops
any left). tools/asc-submit.py reads the released version's section;
a version without one gets a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.22.0

### zh-Hant
- 用麥克風聽寫時，麥克風按鈕會變成錄音膠囊：停止鍵、錄了多久，還有跟著你說話起伏的點點
- Bot 工作中時點一下狀態列，就能即時看它的思考過程和每個步驟
- 麥克風旁的向上箭頭可以選聽寫要用裝置內建語音，還是用你自己的 OpenAI 或 Gemini 金鑰
- iPad 接實體鍵盤時，在輸入框按 Return 會換行，⌘↩ 送出
- 群組裡被提及的 bot 若正在別的對話忙，群組會顯示它在忙，忙完就回來回答

### en-US
- While you dictate, the mic turns into a recording pill: a stop button, how long you've been talking, and dots that follow your voice
- While a bot works, tap its status line to watch its thinking and each step live
- The arrow beside the mic picks how dictation hears you: on-device speech, or your own OpenAI or Gemini key
- On iPad with a hardware keyboard, Return starts a new line in the message box and ⌘↩ sends
- In a group, a bot you mention that's busy in another chat now says so, and answers once it's done
