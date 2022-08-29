use serenity::{model::{prelude::ResumedEvent, gateway::Ready}, framework::standard::macros::hook};


use std::collections::HashSet;

use serenity::http::Http;



use std::fs;
use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::StandardFramework;
use serenity::client::bridge::gateway::ShardManager;
use tracing::{debug, error, info, instrument};

use crate::counting::*;
use crate::owner::*;
use crate::utilcmds::*;

mod counting;
mod owner;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, mut msg: Message) {
        count_handler(&ctx, &mut msg).await;
    }


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
        .group(&COUNTING_GROUP)
        .group(&OWNER_GROUP)
        .group(&UTILCMDS_GROUP);

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