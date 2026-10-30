<!--
Google Play release notes for the Android app. Write it in the change that bumps the version:
a `## <version>` section with one `### <locale>` per Play language (zh-Hant, published as zh-TW;
en-US), listing what Android users notice, each under 500 characters (Play's limit;
tools/play-submit.py cuts longer notes at the last whole line). tools/play-submit.py reads the
released version's section; a version without one gets the PRs' "What's New: Android" bullets,
else a generic "Bug fixes and improvements" line. Delete old sections.
-->

## 2.33.0

### zh-Hant
- 任何 Bot 都能當主助理：長按 Bot 選「設為主助理」，通話時它會先開口。
- 主助理可以像其他 Bot 一樣拖曳排序、釘選或移到分組，不需要時也可以直接刪除。
- Bot 選單更簡潔：不再需要手動開新的工作階段，Codync 會在需要時自動接續。

### en-US
- Any bot can be your lead: touch and hold it and choose Make lead agent. It speaks first on calls.
- Drag, pin or move the lead into a section like any bot, or delete it if you don't need one.
- A simpler bot menu: no more New session, since Codync starts a fresh one on its own when a conversation needs it.
