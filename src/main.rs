use serenity::model::prelude::MessageId;
use serenity::futures::StreamExt;
use serenity::model::prelude::MessagesIter;
use serenity::model::prelude::ResumedEvent;
use serenity::model::gateway::Ready;
use std::collections::HashSet;
use serenity::http::Http;
use serenity::framework::standard::Args;
use serenity::model::prelude::Activity;
use std::time::Instant;
use std::fs;
use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use tracing::{debug, error, info, instrument};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
    }

    // For instrument to work, all parameters must implement Debug.
    //
    // Handler doesn't implement Debug here, so we specify to skip that argument.
    // Context doesn't implement Debug either, so it is also skipped.
    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        // Log at the DEBUG level.
        //
        // In this example, this will not show up in the logs because DEBUG is
        // below INFO, which is the set debug level.
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

#[hook]
// instrument will show additional information on all the logs that happen inside
// the function.
//
// This additional information includes the function name, along with all it's arguments
// formatted with the Debug impl.
// This additional information will also only be shown if the LOG level is set to `debug`
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!("Got command '{}' by user '{}'", command_name, msg.author.name);
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    true
}


#[group]
#[commands(ping, replyping, say, time, check)]
struct General;



#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = fs::read_to_string(".token").expect("Failed to read bot token from disk");
    
    let http = Http::new(&token);
    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("ß")) // set the bot's prefix
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    
    

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
        error!("Client error: {:?}", why);
    }
}


const ERR_SHARD_MGR: &str = "Failed to obtain shard manager";
const ERR_SHARD: &str = "Failed to obtain shard";
const ERR_DURATION: &str = "Failed to obtain shard runner latency";


pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}



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



#[command]
#[owners_only]
#[aliases(ay)]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    let content = args.remains().unwrap_or("​");
    msg.channel_id.send_message(ctx, |m| m.content(content)).await?;
    
    Ok(())
}

#[non_exhaustive]
pub struct MessageCreateEvent {
    pub message: Message,
}