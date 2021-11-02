
use aws_lambda_events::event::sqs::SqsEvent;
use crate::library::aws_client::AWSClient;
use crate::library::aws_client::AWSConfig;
use lambda_runtime::{handler_fn, Error, Context};
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod dtos;
mod errors;
mod library;
mod models;
mod queries;
use library::lambda::handler::execute;

#[tokio::main]
async fn main() -> Result<(), Error> {
  // required to enable CloudWatch error logging by the runtime
  SimpleLogger::new()
    .with_level(LevelFilter::Info)
    .init()
    .unwrap();

  let config = aws_config::load_from_env().await;
  let aws_client = AWSConfig::set_config(config);
  let dynamo_db_client = aws_client.dynamo_client();
  let sqs_client = aws_client.sqs_client();
  let aws_client = AWSClient {
    dynamo_db_client,
    sqs_client,
  };

  lambda_runtime::run(handler_fn(|event: SqsEvent, ctx: Context| {
        execute(&aws_client, event, ctx)
    })) 
    .await?;

  Ok(())
}
