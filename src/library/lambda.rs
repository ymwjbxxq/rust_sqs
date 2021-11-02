pub mod handler {
  use futures::future::join_all;
  use crate::dtos::request::Request;
  use crate::models::product::Product;
  use crate::queries::add_product_query::AddProduct;
  use crate::queries::add_product_query::AddQuery;
  use crate::queries::get_product_by_id_query::GetById;
  use crate::queries::get_product_by_id_query::GetByIdQuery;
  use crate::AWSClient;
  use aws_lambda_events::event::sqs::SqsEvent;
  // use async_std::task;
  use lambda_runtime::{Context, Error};
  use uuid::Uuid;
  use std::sync::Arc;

  pub async fn execute(client: &AWSClient, event: SqsEvent, _ctx: Context) -> Result<Option<Product>, Error> {
    log::info!("EVENT {:?}", event);
    let mut tasks = Vec::with_capacity(event.records.len());
    let shared_client = Arc::from(client.clone());
    for record in event.records.into_iter() {
      let _shared_client = Arc::clone(&shared_client);
      tasks.push(tokio::spawn(async move {
        if let Some(body) = &record.body {
          let request: Request = serde_json::from_str(&body).unwrap();
          if let Some(pk) = request.pk {
            send_to_sqs(&_shared_client, &pk)
            .await
            .map_or_else(|e| log::error!("Error from send_to_sqs {:?}", e), |_| ());
          } else {
            add(&_shared_client, request)
            .await
            .map_or_else(|e| log::error!("Error from add {:?}", e), |_| ());
          }
        } else {
          log::error!("Empty body {:?}", record);
        }
      }))
    }

    join_all(tasks).await;

    Ok(None)
  }

  async fn send_to_sqs(client: &AWSClient, pk: &str) -> Result<(), Error> {
    let product = GetById::new()
      .await
      .execute(&client.dynamo_db_client, &pk)
      .await?;

    // send to sqs
    let sqs_url = std::env::var("OUTPUT_SQS").expect("OUTPUT_SQS must be set");
    let msg_body = serde_json::to_string(&product).unwrap();
    let result = client.sqs_client
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
}
