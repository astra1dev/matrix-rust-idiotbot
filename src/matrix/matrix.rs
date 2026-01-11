use crate::Idiots;
use crate::matrix::commands::{cmd_idiot, cmd_stats};

use matrix_sdk::{
    Client, Room,
    config::SyncSettings,
    event_handler::Ctx,
    ruma::events::{
        reaction::OriginalSyncReactionEvent,
        room::{
            member::StrippedRoomMemberEvent,
            message::{
                FormattedBody, MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
            },
            redaction::OriginalSyncRoomRedactionEvent,
        },
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn send_message(msg: &str, room: &Room) {
    let content = RoomMessageEventContent::text_plain(msg);
    if let Err(err) = room.send(content).await {
        println!("Failed to send message to {}: {}", room.room_id(), err);
    };
}

pub async fn send_message_markdown(msg: &str, room: &Room) {
    let content = RoomMessageEventContent::text_markdown(msg);
    if let Err(err) = room.send(content).await {
        println!("Failed to send message to {}: {}", room.room_id(), err);
    };
}

pub async fn login_and_sync(
    homeserver_url: String,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    // Note that when encryption is enabled, you should use a persistent store to be
    // able to restore the session with a working encryption setup.
    // See the `persist_session` example.
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .build()
        .await?;

    client
        .matrix_auth()
        .login_username(username, password)
        //.initial_device_display_name("autojoin bot")
        .await?;

    println!("logged in as {username}");

    let idiots = Arc::new(Mutex::new(Idiots {
        hash_map: HashMap::new(),
    }));
    client.add_event_handler_context(idiots);

    client.add_event_handler(on_stripped_state_member);
    client.add_event_handler(on_room_message);
    client.add_event_handler(on_reaction);
    client.add_event_handler(on_redaction);

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    client.sync(SyncSettings::default()).await?;

    Ok(())
}

pub async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) {
    if room_member.state_key != client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        println!("Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.join().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            eprintln!(
                "Failed to join room {} ({err:?}), retrying in {delay}s",
                room.room_id()
            );

            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                eprintln!("Can't join room {} ({err:?})", room.room_id());
                break;
            }
        }
        println!("Successfully joined room {}", room.room_id());
    });
}

pub async fn on_room_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    ctx: Ctx<Arc<Mutex<Idiots>>>,
) {
    let Ctx(idiots) = ctx;
    let mut idiots = idiots.lock().await;

    println!("Received a message: {:?}\n", ev);

    if let MessageType::Text(ref text_content) = ev.content.msgtype {
        println!("{}", text_content.body);
        let text = text_content.body.split_whitespace().collect::<Vec<&str>>();
        if let Some(cmd_prefix) = text.first() {
            if cmd_prefix == &".idiot" {
                cmd_idiot(&ev, &room, &mut idiots, text).await;
            } else if cmd_prefix == &".stats" {
                cmd_stats(&room, &mut idiots).await;
            }
        }
    }
}

pub async fn on_reaction(
    ev: OriginalSyncReactionEvent,
    room: Room,
    client: Client,
    ctx: Ctx<Arc<Mutex<Idiots>>>,
) {
    let Ctx(idiots) = ctx;
    let mut idiots = idiots.lock().await;

    println!("Received a reaction event {:?}", ev);
    println!("Emoji: {:?}", ev.content.relates_to.key);

    if ev.content.relates_to.key == "🔥" {
        // Check if the message that was reacted to is in the event id hashmap
        if let Some(i) = idiots
            .hash_map
            .values_mut()
            .find_map(|e| e.get_mut(&ev.content.relates_to.event_id))
        {
            // Block users upvoting their own .idiot report
            if ev.sender == i.2 {
                println!("{} upvoted their own idiot report, ignoring.", ev.sender);
            } else {
                i.1 += 1;
                send_message(
                    &format!(
                        "New amount ({}) for reason: {}\n- Event ID: {}",
                        i.1, i.0, ev.event_id
                    ),
                    &room,
                )
                .await;
            }
        }
    }
}

pub async fn on_redaction(
    ev: OriginalSyncRoomRedactionEvent,
    room: Room,
    client: Client,
    ctx: Ctx<Arc<Mutex<Idiots>>>,
) {
    let Ctx(idiots) = ctx;
    let mut idiots = idiots.lock().await;

    println!("Received a redaction event {:?}", ev);
    let redaction = ev.content.redacts.unwrap();
    if let Some(i) = idiots
        .hash_map
        .values_mut()
        .find_map(|e| e.get_mut(&redaction))
    {
        i.1 -= 1;
        send_message(
            &format!(
                "New amount ({}) for reason: {}\n- Event ID: {}",
                i.1, i.0, ev.event_id
            ),
            &room,
        )
        .await;
    }
}
