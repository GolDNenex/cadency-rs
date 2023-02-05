use crate::{
    command::{command_not_implemented, setup_commands},
    response::{ResponseBuilder, ResponseTiming},
    utils,
    utils::set_bot_presence,
    CadencyError,
};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{application::interaction::Interaction, event::ResumedEvent, gateway::Ready},
};

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
            let cmd_target = utils::get_commands(&ctx)
                .await
                .into_iter()
                .find(|cadency_command| cadency_command.name() == command.data.name.as_str());

            if let Some(cmd) = cmd_target {
                info!("⚡ Execute '{}' command", cmd.name());
                if cmd.deferred() {
                    ResponseBuilder::new(ResponseTiming::DeferredInfo)
                        .build()
                        .expect("Failed to build response")
                        .submit(&ctx, &mut command)
                        .await
                        .expect("Unable to submit deferred info");
                }
                if let Err(command_error) = cmd.execute(&ctx, &mut command).await {
                    error!("❌ Command execution failed: {command_error:?}");
                    let mut error_res_builder = ResponseBuilder::default();
                    if cmd.deferred() {
                        error_res_builder.timing(ResponseTiming::Deferred);
                    } else {
                        error_res_builder.timing(ResponseTiming::Instant);
                    }
                    match command_error {
                        CadencyError::Command { message } => {
                            error_res_builder.message(Some(message));
                            error_res_builder.build()
                        }
                        _ => error_res_builder
                            .message(Some("**Oops! Something went terrible wrong.**".to_string()))
                            .build(),
                    }
                    .expect("Unable to build error response")
                    .submit(&ctx, &mut command)
                    .await
                    .map_err(|err| {
                        error!("❌ Fatal error! Is discord down? {:?}", err);
                    })
                    .expect("Unable to send error response");
                } else {
                    info!("✅ Command '{}' was successful", cmd.name())
                }
            } else {
                command_not_implemented(&ctx, &command)
                    .await
                    .expect("Failed to submit not-implemented error");
            }
        };
    }
}
