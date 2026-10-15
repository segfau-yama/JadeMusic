use async_trait::async_trait;
use poise::serenity_prelude as serenity;
use songbird::{
    events::{CoreEvent, Event, EventContext, EventHandler as VoiceEventHandler},
    Call, Songbird,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::services::clear_queue;

/// join / auto-join 直後に一度だけ呼び出し、CoreEvent ハンドラを登録する。
pub async fn register(
    handler_lock: &Arc<Mutex<Call>>,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    cache: Arc<serenity::Cache>,
) {
    let bot_id = cache.current_user().id;
    let evt = VoiceHandler {
        manager,
        guild_id,
        cache,
        bot_id,
    };
    let mut handler = handler_lock.lock().await;
    handler.add_global_event(CoreEvent::DriverDisconnect.into(), evt.clone());
    handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt);
}

#[derive(Clone)]
struct VoiceHandler {
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    cache: Arc<serenity::Cache>,
    bot_id: serenity::UserId,
}

#[async_trait]
impl VoiceEventHandler for VoiceHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            // ボットが切断された (強制退出・ネットワーク断など) のでキューを空にする
            EventContext::DriverDisconnect(_) => {
                clear_queue(&self.manager, self.guild_id).await;
            }

            // 同チャンネルの誰かが切断した ─ 人間がいなくなったら自動退出
            EventContext::ClientDisconnect(_) => {
                let handler_lock = self.manager.get(self.guild_id)?;
                let bot_channel_id = {
                    let handler = handler_lock.lock().await;
                    handler.current_channel()
                };
                let bot_channel_id = bot_channel_id?;
                let has_humans = {
                    let guild = self.cache.guild(self.guild_id)?;
                    guild.voice_states.values().any(|vs| {
                        vs.user_id != self.bot_id
                            && vs.channel_id.is_some_and(|ch| ch.get() == bot_channel_id.0.get())
                    })
                };
                if !has_humans {
                    clear_queue(&self.manager, self.guild_id).await;
                    let _ = self.manager.remove(self.guild_id).await;
                }
            }

            _ => {}
        }
        None
    }
}
