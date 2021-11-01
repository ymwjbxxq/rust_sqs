pub mod handler {
  use crate::dtos::request::Request;
  use crate::models::product::Product;
  use crate::queries::add_product_query::AddProduct;
  use crate::queries::add_product_query::AddQuery;
  use crate::queries::get_product_by_id_query::GetById;
  use crate::queries::get_product_by_id_query::GetByIdQuery;
  use crate::AWSClient;
  use aws_lambda_events::event::sqs::SqsEvent;
  use futures::stream::{self, StreamExt};
  use lambda_runtime::{Context, Error};
  use uuid::Uuid;

  pub async fn execute(client: &AWSClient, event: SqsEvent, _ctx: Context) -> Result<Option<Product>, Error> {
    log::info!("EVENT {:?}", event);
    let mut records = stream::iter(event.records); // convert array in streams to use async

    log::info!("records {:?}", records);
    // processing one element at a time
    while let Some(record) = records.next().await {
      if let Some(body) = record.body {
        let request: Request = serde_json::from_str(&body).unwrap();
        if let Some(pk) = request.pk {
          send_to_sqs(&client, &pk).await?;
        } else {
          add(&client, request).await?;
        }
      } else {
        log::error!("empty body");
      }
    }

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
}

