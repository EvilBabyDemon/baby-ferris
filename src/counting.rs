use std::str::FromStr;

use serenity::{framework::standard::{macros::{group, command}, Args, CommandResult}, model::prelude::{ChannelId, Message}, prelude::{Context, TypeMapKey}};


#[group]
#[commands(count)]
struct Counting;


pub const ERR_NO_COUNTING: &str = "No counting setup found in bot";

pub struct Count {
    listen: u64,
    channel: ChannelId,
    last_num: u64,
}

impl TypeMapKey for Count {
    type Value = Count;
}


pub async fn count_handler(ctx: &Context, msg: &Message) {
    let mut data = ctx.data.write().await;
    let mut counting = data.get_mut::<Count>().expect(ERR_NO_COUNTING);
    if msg.channel_id == counting.channel && msg.author.id == counting.listen {
        if let Ok(value) = msg.content.parse::<u64>() {
            let new = value + 1;
            counting.last_num = new;
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", new)).await {
                data.downgrade();
                println!("{e}");
            }
        }
    }

    let num = msg.content.parse::<u64>().unwrap();
    let err;
    
    if num == 1 && 1 == *msg.author.id.as_u64() {
        err = msg.channel_id.send_message(ctx, |m| m.content(num + 1)).await;
        if err.is_err() {
            msg.channel_id.send_message(ctx, |m| m.content(num + 1)).await;
        }
    }

}


#[command]
#[description("Sets the bot counting following a given user in a given channel")]
#[usage("#channel @user")]
#[num_args(2)]
async fn count(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut setup = data.get_mut::<Count>().expect(ERR_NO_COUNTING);
    setup.channel = parse_id(&mut args)?;
    setup.listen = parse_id(&mut args)?;
    msg.reply(
        ctx,
        format!(
            "Counting in channel <#{}> following user <@{}>",
            setup.channel, setup.listen
        ),
    )
    .await?;
    Ok(())
}

fn parse_id<T: FromStr>(args: &mut Args) -> Result<T, &str> {
    match args.single::<T>() {
        Ok(value) => Ok(value),
        Err(_) => {
            let input = args
                .single::<String>()
                .expect("Failed to read string argument");
            let len = input.len();
            if len < 4 {
                return Err("Invalid ID");
            }
            match (&input[2..len - 1]).parse::<T>() {
                Ok(id) => Ok(id),
                Err(_) => Err("Failed to parse ID"),
            }
        }
    }
}



