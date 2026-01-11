use crate::matrix::matrix::send_message;
use crate::{Idiots, Reason, Upvotes, Username};

use matrix_sdk::Room;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use std::collections::HashMap;
use tokio::sync::MutexGuard;

pub async fn cmd_idiot(
    ev: &OriginalSyncRoomMessageEvent,
    room: &Room,
    idiots: &mut MutexGuard<'_, Idiots>,
    text: Vec<&str>,
) {
    if let Some(name) = text.get(1).cloned() {
        let reason = text.into_iter().skip(2).collect::<Vec<&str>>().join(" ");
        if reason.is_empty() {
            send_message(
                &format!("Failed to expose {}! A reason is required.", name),
                &room,
            )
            .await;
        } else {
            match idiots.hash_map.get_mut(name) {
                None => {
                    // The user was not yet in the list of idiots, create a new entry
                    let mut new_hash_map = HashMap::new();
                    new_hash_map.insert(
                        ev.event_id.clone(),
                        (reason.to_string(), 1, ev.sender.clone().to_string()),
                    );
                    idiots.hash_map.insert(name.to_owned(), new_hash_map);
                    send_message(
                        &format!(
                            "🔥 {} was exposed as an idiot!\n- Reason: {}\n- Reported by: {}\n- Event ID: {}",
                            name, reason, ev.sender, ev.event_id
                        ),
                        &room,
                    )
                        .await;
                }
                Some(reason_hashmap) => {
                    // The user was found in the list of idiots
                    reason_hashmap.insert(
                        ev.event_id.clone(),
                        (reason.to_string(), 1, ev.sender.clone().to_string()),
                    );
                    send_message(
                        &format!(
                            "🔥 {} was (once again) exposed as an idiot!\n- Reason: {}\n- Reported by: {}\n- Event ID: {}",
                            name, reason, ev.sender, ev.event_id
                        ),
                        &room,
                    )
                        .await;
                }
            }
        }
    } else {
        send_message("Failed to expose idiot! A name is required.", &room).await;
    }
}

pub async fn cmd_stats(room: &Room, idiots: &mut MutexGuard<'_, Idiots>) {
    if idiots.hash_map.is_empty() {
        send_message("No idiots have been exposed yet!", &room).await;
        return;
    }

    // Most liked report(s) leaderboard
    let mut all_reports = idiots
        .hash_map
        .iter()
        .flat_map(|e| {
            e.1.values()
                .map(|f| (e.0, f.0.clone(), f.1))
                .collect::<Vec<(&Username, Reason, Upvotes)>>()
        })
        .collect::<Vec<(&Username, Reason, Upvotes)>>();
    all_reports.sort_by(|b, a| a.2.cmp(&b.2)); // sort descending by upvotes

    if let Some(top_report) = all_reports.first() {
        let top_upvotes = top_report.2;
        let tied_reports: Vec<_> = all_reports
            .iter()
            .take_while(|report| report.2 == top_upvotes)
            .collect();

        if tied_reports.len() == 1 {
            send_message(
                &format!(
                    "**Most upvoted Report:**\n- User: {}\n- Reason: {}\n- Upvotes: {}",
                    top_report.0, top_report.1, top_report.2
                ),
                &room,
            )
            .await;
        } else {
            let msg = tied_reports
                .iter()
                .map(|e| format!("- User: {}\n  Reason: {}\n  Upvotes: {}", e.0, e.1, e.2))
                .collect::<Vec<_>>()
                .join("\n");
            send_message(&format!("**Most upvoted Reports (tied):**\n{}", msg), &room).await;
        }
    }

    // Idiot leaderboard (who has the most cumulative upvotes)
    let mut idiot_totals = idiots
        .hash_map
        .iter()
        .map(|(name, events)| (name, events.values().map(|v| v.1).sum()))
        .collect::<Vec<(&Username, Upvotes)>>();
    idiot_totals.sort_by(|b, a| a.1.cmp(&b.1)); // sort descending by upvotes

    if let Some(top_idiot) = idiot_totals.first() {
        let top_upvotes = top_idiot.1;
        let tied_idiots: Vec<_> = idiot_totals
            .iter()
            .take_while(|idiot| idiot.1 == top_upvotes)
            .collect();

        if tied_idiots.len() == 1 {
            send_message(
                &format!(
                    "**Biggest Idiot:**\n- User: {}\n- Total Upvotes: {}",
                    top_idiot.0, top_idiot.1
                ),
                &room,
            )
            .await;
        } else {
            let msg = tied_idiots
                .iter()
                .map(|i| format!("- User: {}\n  Total Upvotes: {}", i.0, i.1))
                .collect::<Vec<_>>()
                .join("\n");
            send_message(&format!("**Biggest Idiots (tied):**\n{}", msg), &room).await;
        }
    }
}
