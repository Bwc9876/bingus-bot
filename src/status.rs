use log::debug;
use twilight_gateway::MessageSender;
use twilight_model::gateway::{
    payload::outgoing::UpdatePresence,
    presence::{Activity, ActivityType, Status},
};

use crate::{brain::Brain, prelude::*};

pub fn update_status(brain: &Brain, sender: &MessageSender) -> Result {
    let words = brain.word_count();

    let activity = Activity {
        application_id: None,
        assets: None,
        buttons: Vec::new(),
        created_at: None,
        details: None,
        emoji: None,
        flags: None,
        id: None,
        instance: None,
        kind: ActivityType::Custom,
        name: "Bingus".to_string(),
        party: None,
        secrets: None,
        state: Some(format!("I know {words} words!")),
        timestamps: None,
        url: None,
    };

    let status = UpdatePresence::new(vec![activity], false, None, Status::Online)
        .context("Failed to make status")?;

    sender
        .command(&status)
        .context("Failed to send to gateway")?;

    debug!("Sent status update");
    Ok(())
}
