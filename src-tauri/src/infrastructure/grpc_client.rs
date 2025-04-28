use tonic::transport::Channel;

use crate::pb::tempgrpcd::tempgrpcd_client::TempgrpcdClient;

async fn new(endpoint: &'static str) -> Result<TempgrpcdClient<Channel>, Box<dyn std::error::Error>> {
    let client = TempgrpcdClient::connect(endpoint).await?;

    Ok(client)
}
