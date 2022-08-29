use serenity::{prelude::Context, model::prelude::{Message, Activity}, framework::standard::{Args, CommandResult, macros::{group, command}}};


#[group]
#[commands(say, activity)]
struct Owner;


#[command]
#[owners_only]
#[aliases(ay)]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    let content = args.remains().unwrap_or("​");
    msg.channel_id.send_message(ctx, |m| m.content(content)).await?;
    
    Ok(())
}

#[command]
#[owners_only]
async fn activity(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    
    let selection = args.single::<String>().unwrap();

    let status = args.remains().unwrap();

    ctx.set_activity(
        match selection.as_str() {
            "playing" => Activity::playing(status),
            "watching" => Activity::watching(status),
            "status" => Activity::listening(status),
            "competing" => Activity::competing(status),
            _ => Activity::playing(status)
        }
    ).await;
    
    Ok(())
}