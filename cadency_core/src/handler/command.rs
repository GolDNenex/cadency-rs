use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{application::interaction::Interaction, event::ResumedEvent, gateway::Ready},
};

use crate::{
    command::{command_not_implemented, setup_commands},
    utils,
};

use crate::utils::set_bot_presence;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        info!("🚀 Start Cadency Discord Bot");
        set_bot_presence(&ctx).await;
        info!("⏳ Started to submit commands, please wait...");
        match setup_commands(&ctx).await {
            Ok(_) => info!("✅ Application commands submitted"),
            Err(err) => error!("❌ Failed to submit application commands: {:?}", err),
        };
    }

    async fn resume(&self, _ctx: Context, _: ResumedEvent) {
        debug!("🔌 Reconnect to server");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(mut command) = interaction {
            let cadency_commands = utils::get_commands(&ctx).await;
            let command_name = command.data.name.as_str();
            let cmd_target = cadency_commands
                .iter()
                .find(|cadency_command| cadency_command.name() == command_name);
            let cmd_execution = match cmd_target {
                Some(target) => target.execute(&ctx, &mut command).await,
                None => command_not_implemented(&ctx, command).await,
            };
            if let Err(execution_err) = cmd_execution {
                error!("❌ Command execution failed: {execution_err:?}");
            }
        };
    }
}
