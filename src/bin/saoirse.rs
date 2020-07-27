use dotenv::dotenv;

use saoirse::{api::Api, Context};

#[tokio::main]
async fn main() -> Result<(), saoirse::error::Error> {
    dotenv().ok();

    let ctx = Context::new().await?;
    let api = Api::new(ctx);
    let addr = ([127, 0, 0, 1], 8080);

    api.serve(addr).await;

    Ok(())
}
