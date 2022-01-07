use crate::error::Error;
use aws_sdk_dynamodb::model::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
  pub pk: String,
  pub name: String,
  pub price: f64,
}

enum ValueType {
  N,
  S,
}

impl Product {
  pub fn to_dynamodb(&self) -> HashMap<String, AttributeValue> {
    let mut retval = HashMap::new();
    retval.insert("pk".to_owned(), AttributeValue::S(self.pk.clone()));
    retval.insert("name".to_owned(), AttributeValue::S(self.name.clone()));
    retval.insert(
      "price".to_owned(),
      AttributeValue::N(format!("{:}", self.price)),
    );

    retval
  }

  pub fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error> {
    Ok(Product {
      pk: Product::get_key("pk", ValueType::S, &value)?,
      name: Product::get_key("name", ValueType::S, &value)?,
      price: Product::get_key("price", ValueType::N, &value)?
        .parse::<f64>()
        .unwrap(),
    })
  }

  fn get_key(
    key: &str,
    t: ValueType,
    item: &HashMap<String, AttributeValue>,
  ) -> Result<String, Error> {
    let v = item
      .get(key)
      .ok_or_else(|| Error::InternalError(format!("Missing '{}'", key)))?;

    Ok(match t {
      ValueType::N => v.as_n()?.to_owned(),
      ValueType::S => v.as_s()?.to_owned(),
    })
  }
}
