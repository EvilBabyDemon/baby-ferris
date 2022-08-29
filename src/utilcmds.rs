use serenity::{prelude::Context, model::prelude::Message, framework::standard::{Args, CommandResult, macros::{group, command}}};


#[group]
#[commands(ping, replyping, time, check)]
struct utilcmds;

#[command]
#[aliases(ing)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let pingstr = "Pong!".to_owned();
    
    let data = ctx.data.write().await;
    let heartbeat = match data.get::<ShardManagerContainer>() {
        Some(manager) => {
            let mgr = manager.lock().await;
            let runners = mgr.runners.lock().await;
            match runners.get(&ShardId(ctx.shard_id)) {
                Some(runner) => match runner.latency {
                    Some(duration) => format!("Hearbeat: {}ms", duration.as_millis()),
                    None => String::from(ERR_DURATION),
                },
                None => String::from(ERR_SHARD),
            }
        }
        None => String::from(ERR_SHARD_MGR),
    };

    let time = Instant::now();
    let mut ping = msg.channel_id.send_message(ctx, |m| m.content(&pingstr)).await?;
    let rtt = time.elapsed().as_millis();
    ping.edit(ctx, |m| m.content(pingstr + " " + &rtt.to_string() + "ms" + "\n" + &heartbeat)).await?; 
   
    Ok(())
}

#[command]
async fn replyping(ctx: &Context, msg: &Message) -> CommandResult {
    let pingstr = "Pong!".to_owned();
    let time = Instant::now();
    let mut ping = msg.reply(ctx, &pingstr).await?;
    let rtt = time.elapsed().as_millis();
    ping.edit(ctx, |m| m.content(pingstr + " " + &rtt.to_string() + "ms")).await?;
    
    Ok(())
}


#[command]
#[aliases(ime)]
async fn time(ctx: &Context, msg: &Message) -> CommandResult {
    
    let msgtime = msg.id.created_at();
    let time = Instant::now();
    let mut sent = msg.channel_id.send_message(ctx, |m| m.content("a")).await?;
    let rtt_selftimed = time.elapsed().as_millis();
    let rtt_discord = sent.id.created_at().timestamp_millis()-msgtime.timestamp_millis();
    sent.edit(ctx, |m| m.content(rtt_discord.to_string() + "ms calculated with snowflake id\n" + &rtt_selftimed.to_string() + "ms selftimed ping")).await?;

    Ok(())
}

#[command]
#[aliases(eck)]
async fn check(ctx: &Context, msg: &Message) -> CommandResult {
    let replied = &msg.message_reference.as_ref().unwrap().message_id.unwrap().as_u64().clone();    
    let mut messages = MessagesIter::<Http>::stream(&ctx, msg.channel_id).boxed();

    let mut lastmessage = 0;
    while let Some(message_result) = messages.next().await {
        let message_result = message_result.unwrap();
        let result_id = message_result.id.as_u64();
        if replied == result_id {
            break;
        }
        lastmessage = result_id.clone();
    }


    let id = MessageId::from(lastmessage);
    
    let calcping = id.created_at().timestamp_millis() - MessageId::from(*replied).created_at().timestamp_millis();
    msg.reply(ctx, calcping.to_string() + "ms").await?;

    Ok(())
}