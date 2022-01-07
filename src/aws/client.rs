pub struct AWSConfig {
  config: aws_types::config::Config,
}

impl AWSConfig {
   pub fn set_config(config: aws_types::config::Config) -> Self {
    Self { 
      config: config 
    }
  }

  pub fn dynamo_client(&self) -> aws_sdk_dynamodb::Client {
    let dynamo_db_client = aws_sdk_dynamodb::Client::new(&self.config);
    return dynamo_db_client;
  }

  pub fn sqs_client(&self) -> aws_sdk_sqs::Client {
    let sqs_client = aws_sdk_sqs::Client::new(&self.config);
    return sqs_client;
  }
}

#[derive(Clone)]
pub struct AWSClient {
  pub dynamo_db_client: aws_sdk_dynamodb::Client,
  pub sqs_client: aws_sdk_sqs::Client,
}
