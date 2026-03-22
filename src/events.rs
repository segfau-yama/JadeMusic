use poise::serenity_prelude as serenity;
use crate::{Data, Error};
use crate::services::music::clear_queue;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            let guild_id = match new.guild_id {
                Some(id) => id,
                None => return Ok(()),
            };

            let manager = match songbird::get(ctx).await {
                Some(m) => m,
                None => return Ok(()),
            };

            let bot_id = ctx.cache.current_user().id;

            if new.user_id == bot_id && new.channel_id.is_none() {
                clear_queue(&manager, guild_id).await;
                return Ok(());
            }

            let handler_lock = match manager.get(guild_id) {
                Some(h) => h,
                None => return Ok(()),
            };

            let bot_channel_id = {
                let handler = handler_lock.lock().await;
                handler.current_channel()
            };

            let bot_channel_id = match bot_channel_id {
                Some(id) => id,
                None => return Ok(()),
            };

            let affected_channel = old
                .as_ref()
                .and_then(|vs| vs.channel_id)
                .or(new.channel_id);
            let affected_channel = match affected_channel {
                Some(ch) => ch,
                None => return Ok(()),
            };
            if affected_channel.get() != bot_channel_id.0.get() {
                return Ok(());
            }

            let human_count = {
                let guild = match ctx.cache.guild(guild_id) {
                    Some(g) => g,
                    None => return Ok(()),
                };
                guild
                    .voice_states
                    .values()
                    .filter(|vs| {
                        vs.channel_id
                            .map(|ch| ch.get() == bot_channel_id.0.get())
                            .unwrap_or(false)
                    })
                    .filter(|vs| vs.user_id != bot_id)
                    .count()
            };

            if human_count == 0 {
                clear_queue(&manager, guild_id).await;
                let _ = manager.remove(guild_id).await;
            }
        }

        _ => {}
    }

    Ok(())
}
