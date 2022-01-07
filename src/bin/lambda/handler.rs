use aws_lambda_events::event::sqs::SqsEvent;
use futures::future::join_all;
use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use rust_sqs::aws::client::AWSClient;
use rust_sqs::aws::client::AWSConfig;
use rust_sqs::dtos::request::Request;
use rust_sqs::models::product::Product;
use rust_sqs::queries::add_product_query::AddProduct;
use rust_sqs::queries::add_product_query::AddQuery;
use rust_sqs::queries::get_product_by_id_query::GetById;
use rust_sqs::queries::get_product_by_id_query::GetByIdQuery;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
  // Initialize service
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

pub async fn execute(client: &AWSClient, event: SqsEvent, _ctx: Context) -> Result<(), Error> {
  log::info!("EVENT {:?}", event);
  let mut tasks = Vec::with_capacity(event.records.len());
  let shared_client = Arc::from(client.clone());
  for record in event.records.into_iter() {
    let shared_client = Arc::clone(&shared_client);
    tasks.push(tokio::spawn(async move {
      if let Some(body) = &record.body {
        let request: Request = serde_json::from_str(&body).unwrap();
        if let Some(pk) = request.pk {
          send_to_sqs(&shared_client, &pk)
            .await
            .map_or_else(|e| log::error!("Error from send_to_sqs {:?}", e), |_| ());
        } else {
          add(&shared_client, request)
            .await
            .map_or_else(|e| log::error!("Error from add {:?}", e), |_| ());
        }
      } else {
        log::error!("Empty body {:?}", record);
      }
    }))
  }

  join_all(tasks).await;

  Ok(())
}

async fn send_to_sqs(client: &AWSClient, pk: &str) -> Result<(), Error> {
  let product = GetById::new()
    .await
    .execute(&client.dynamo_db_client, &pk)
    .await?;

  // send to sqs
  let sqs_url = std::env::var("OUTPUT_SQS").expect("OUTPUT_SQS must be set");
  let msg_body = serde_json::to_string(&product).unwrap();
  let result = client
    .sqs_client
    .send_message()
    .queue_url(sqs_url)
    .message_body(msg_body)
    .send()
    .await?;
  log::info!("SQS sent {:?}", result);

  Ok(())
}

async fn add(client: &AWSClient, request: Request) -> Result<(), Error> {
  let product = Product {
    pk: Uuid::new_v4().to_string(),
    name: request.name.unwrap(),
    price: request.price.unwrap(),
  };
  AddProduct::new()
    .await
    .execute(&client.dynamo_db_client, &product)
    .await?;

  Ok(())
}
