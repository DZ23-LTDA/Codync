use super::*;
use serde_json::json;

fn bot(id: &str, name: &str) -> BotConfig {
    serde_json::from_value(json!({"id": id, "name": name, "backend": "claude", "cwd": "/tmp"})).expect("valid bot")
}

fn said(text: &str) -> Entry {
    Entry {
        id: "e".into(),
        seq: 1,
        bot_id: "g".into(),
        thread_id: None,
        rev: 1,
        kind: "user".into(),
        turn: 1,
        data: json!({"text": text}),
        created_at: 0,
        updated_at: 0,
    }
}

#[test]
fn mentions_pick_responders_and_no_mention_means_everyone() {
    let members = [bot("a", "Alice Chen"), bot("b", "Bob"), bot("c", "尼采課")];
    let ids = |m: &[BotConfig], text: &str| -> Vec<String> {
        responders(m, &[said(text)]).into_iter().map(|b| b.id.clone()).collect()
    };
    assert_eq!(ids(&members, "hi all"), ["a", "b", "c"]);
    assert_eq!(ids(&members, "@bob can you look"), ["b"]);
    assert_eq!(ids(&members, "@alicechen and @Alice"), ["a"]);
    assert_eq!(ids(&members, "@尼采課幫我整理"), ["c"], "CJK names can run into the message");
    assert_eq!(ids(&members, "@bobby?"), ["a", "b", "c"], "a longer word isn't a mention");
    assert_eq!(ids(&members, "@all go"), ["a", "b", "c"]);
}

use crate::api::devices::Caller;
use crate::hub::BotStatus;
use std::path::PathBuf;
use std::time::Duration;

/// A host with agent bots `a` and `b` (the scripted fixture agent) and a group of both.
struct Room {
    hub: Arc<Hub>,
    dir: PathBuf,
    group: String,
}

impl Room {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("codync-group-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Store::open(&dir.join("test.db")).unwrap();
        let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/team_agent.py");
        for id in ["a", "b"] {
            let cwd = dir.join(id);
            std::fs::create_dir_all(&cwd).unwrap();
            let cfg: BotConfig = serde_json::from_value(json!({
                "id": id, "name": id, "backend": "fixture", "cwd": cwd,
                "command": crate::shell::python(&agent), "notify": false,
            }))
            .unwrap();
            store.save_bot(&cfg).unwrap();
        }
        let hub = Hub::new(
            store,
            "test-host".into(),
            crate::remote::identity::Identity::load_or_create(&dir).unwrap(),
            "test-token".into(),
            19222,
        );
        hub.start().unwrap();
        let group = hub
            .create_bot(
                serde_json::from_value(
                    json!({"id": "", "kind": "group", "name": "Crew", "description": "Ship the release", "members": ["a", "b", "a"]}),
                )
                .unwrap(),
            )
            .unwrap();
        let group = group["id"].as_str().unwrap().to_owned();
        Self { hub, dir, group }
    }

    async fn send(&self, text: &str, thread: Option<&str>) -> Entry {
        let sent = crate::api::dispatch(
            &self.hub,
            &Caller::Local,
            "send",
            json!({"botId": self.group, "text": text, "threadId": thread}),
        )
        .await
        .unwrap();
        serde_json::from_value::<serde_json::Value>(sent["entry"].clone())
            .ok()
            .and_then(|e| self.hub.store.entry(e["id"].as_str()?))
            .unwrap()
    }

    fn replies(&self, lane: &Lane) -> Vec<(String, String)> {
        self.hub
            .store
            .messages_after(lane, 0, 100)
            .unwrap()
            .into_iter()
            .filter(|e| e.kind == "agent")
            .map(|e| (e.data["author"].as_str().unwrap().to_owned(), e.data["text"].as_str().unwrap().to_owned()))
            .collect()
    }

    async fn settle(&self) {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                tokio::time::sleep(Duration::from_millis(300)).await;
                let idle = ["a", "b"].iter().all(|b| self.hub.runtime(b).status == BotStatus::Idle);
                if idle && self.hub.bots_json(0).unwrap().iter().all(|b| b["status"] == "idle") {
                    break;
                }
            }
        })
        .await
        .expect("room settles");
    }

    async fn shutdown(self) {
        self.hub.shutdown().await;
        let _ = std::fs::remove_dir_all(self.dir);
    }
}

#[tokio::test]
async fn room_turns_follow_mentions_and_end_when_everyone_passes() {
    let room = Room::new();
    let row = room.hub.store.bot(&room.group).unwrap().unwrap();
    assert_eq!(row.config.members, ["a", "b"], "members are deduplicated");
    let again = room
        .hub
        .create_bot(
            serde_json::from_value(json!({"id": "", "kind": "group", "name": "Dup", "members": ["b", "a"]})).unwrap(),
        )
        .unwrap();
    assert_eq!(again["id"], room.group, "the same bots reopen their group");

    let main = Lane::main(&room.group);
    room.send("hello everyone", None).await;
    room.settle().await;
    // `a` answers the user; `b` then sees a bot spoke last and passes; round two is all passes.
    assert_eq!(room.replies(&main), [("a".to_owned(), "reply: group".to_owned())]);

    room.send("@b your turn", None).await;
    room.settle().await;
    assert_eq!(room.replies(&main).last().unwrap().0, "b", "only the mentioned bot answers");
    assert_eq!(room.replies(&main).len(), 2);
    let listed = room.hub.bot_value_for_test(&room.group);
    assert_eq!(listed["lastMessage"], "b: reply: group");
    let prompts = std::fs::read_to_string(room.dir.join("a/prompts.jsonl")).unwrap();
    assert!(prompts.contains("Ship the release) with the user"), "members are told what the room is for: {prompts}");
    // The members' own chats stay private: the room's entries live in the group.
    assert!(room.hub.store.messages_after(&Lane::main("a"), 0, 100).unwrap().is_empty());
    room.shutdown().await;
}

#[tokio::test]
async fn a_skill_command_reaches_the_members_that_have_the_skill_on() {
    let room = Room::new();
    let pdf = json!([{"id": "pdf", "name": "PDF", "description": "", "source": "custom"}]);
    room.hub.store.kv_set("skills", &pdf.to_string()).unwrap();
    let mut a = room.hub.store.bot("a").unwrap().unwrap().config;
    a.skills = vec!["pdf".into()];
    room.hub.store.save_bot(&a).unwrap();

    room.send("/pdf merge these", None).await;
    room.settle().await;
    let prompts = |id: &str| std::fs::read_to_string(room.dir.join(id).join("prompts.jsonl")).unwrap();
    assert!(prompts("a").contains(r#"User: [Codync: the user picked the \"PDF\" skill"#), "{}", prompts("a"));
    assert!(
        prompts("b").contains("User: /pdf merge these"),
        "a member without it reads what was typed: {}",
        prompts("b")
    );
    room.shutdown().await;
}

#[tokio::test]
async fn threads_stay_out_of_the_main_chat_and_summarize_on_their_root() {
    let room = Room::new();
    let first = room.send("@a start here", None).await;
    room.settle().await;
    room.send("@a in the thread", Some(&first.id)).await;
    room.settle().await;
    let thread = Lane::in_thread(&room.group, &first.id);
    assert_eq!(room.replies(&thread).len(), 1);
    assert_eq!(room.replies(&Lane::main(&room.group)).len(), 1, "the thread's reply isn't in the main chat");
    let summary = room.hub.store.entry(&first.id).unwrap().data["thread"].clone();
    assert_eq!(summary["count"], 2);
    assert_eq!(summary["authors"], json!(["user", "a"]));

    // Threads are read on their own (Slack's): reading the main chat leaves the thread's reply unread.
    let unread = || room.hub.bot_value_for_test(&room.group)["unread"].as_i64().unwrap();
    assert_eq!(summary["unread"], 1);
    assert_eq!(unread(), 2, "the main chat's reply and the thread's");
    let mark = |b: serde_json::Value| crate::api::dispatch(&room.hub, &Caller::Local, "markRead", b);
    mark(json!({"botId": room.group})).await.unwrap();
    assert_eq!(unread(), 1);
    mark(json!({"botId": room.group, "threadId": first.id})).await.unwrap();
    assert_eq!(unread(), 0);
    assert_eq!(room.hub.store.entry(&first.id).unwrap().data["thread"]["unread"], 0);
    assert!(mark(json!({"botId": "a", "threadId": first.id})).await.is_err(), "a thread of another chat");

    // Reactions toggle, and reacting to a read message doesn't make it unread.
    let reply = room.hub.store.messages_after(&Lane::main(&room.group), 0, 100).unwrap();
    let reply = reply.iter().find(|e| e.kind == "agent" && e.data["final"] == true).unwrap();
    let react = |emoji: &str| {
        crate::api::dispatch(&room.hub, &Caller::Local, "react", json!({"entryId": reply.id, "emoji": emoji}))
    };
    react("👍").await.unwrap();
    let reacted = react("✅").await.unwrap();
    assert_eq!(reacted["entry"]["data"]["reactions"], json!(["👍", "✅"]));
    assert_eq!(unread(), 0);
    let undone = react("👍").await.unwrap();
    assert_eq!(undone["entry"]["data"]["reactions"], json!(["✅"]));
    assert!(react("hi").await.is_err(), "only emoji");

    // A thread in a bot's own chat gets a session of its own, told where it branched off.
    let dm =
        crate::api::dispatch(&room.hub, &Caller::Local, "send", json!({"botId": "b", "text": "main"})).await.unwrap();
    room.settle().await;
    let dm_root = dm["entry"]["id"].as_str().unwrap().to_owned();
    crate::api::dispatch(
        &room.hub,
        &Caller::Local,
        "send",
        json!({"botId": "b", "text": "aside", "threadId": dm_root}),
    )
    .await
    .unwrap();
    room.settle().await;
    let prompts = std::fs::read_to_string(room.dir.join("b/prompts.jsonl")).unwrap();
    assert!(prompts.lines().last().unwrap().contains("[Thread]"), "{prompts}");
    let replies = room.hub.store.thread("b", &dm_root, 10).unwrap();
    assert_eq!(replies.iter().filter(|e| e.data["final"] == true).count(), 1);
    assert!(
        crate::api::dispatch(
            &room.hub,
            &Caller::Local,
            "send",
            json!({"botId": "b", "text": "x", "threadId": replies[0].id})
        )
        .await
        .is_err(),
        "threads are flat"
    );
    room.shutdown().await;
}

#[tokio::test]
async fn stop_ends_the_room_turn_before_the_next_speaker() {
    let room = Room::new();
    room.send("BLOCK everyone look", None).await;
    let hub = room.hub.clone();
    tokio::time::timeout(Duration::from_secs(10), async {
        while hub.runtime("a").status != BotStatus::Working {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("a starts");
    crate::api::dispatch(&room.hub, &Caller::Local, "stop", json!({"botId": room.group})).await.unwrap();
    room.settle().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!room.dir.join("b/prompts.jsonl").exists(), "b never got a turn after Stop");
    assert!(room.replies(&Lane::main(&room.group)).iter().all(|(_, t)| t != "reply: group"));
    room.shutdown().await;
}

#[test]
fn pass_is_recognized_loosely() {
    for t in ["(pass)", " (pass) ", "\"(pass)\"", "pass.", ""] {
        assert!(is_pass(t), "{t:?}");
    }
    assert!(!is_pass("I'll pass this to Bob"));
}
