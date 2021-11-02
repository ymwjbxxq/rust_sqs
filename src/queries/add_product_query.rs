use crate::models::product::Product;
use crate::errors::error::Error;
use aws_sdk_dynamodb::Client;
use async_trait::async_trait;

#[async_trait]
pub trait AddQuery {
    async fn new() -> Self;
    async fn execute(&self, client: &Client, product: &Product) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct AddProduct {
  table_name: String,
}

#[async_trait]
impl AddQuery for AddProduct {
  async fn new() -> Self {
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    Self { table_name }
  }

  async fn execute(&self, client: &Client, request: &Product) -> Result<(), Error> {
    log::info!("Adding product");
    let res = client
      .put_item()
      .table_name(&self.table_name)
      .set_item(Some(request.to_dynamodb()))
      .send()
      .await?;
     log::info!("Product added {:?}", res);

    Ok(())
  }
}
