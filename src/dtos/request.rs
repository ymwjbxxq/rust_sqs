use serde::{Deserialize};

#[derive(Deserialize, Debug, Default)]
pub struct Request {
  pub pk: Option<String>,
  pub name: Option<String>,
  pub price: Option<f64>,
}
