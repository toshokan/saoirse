#[tokio::main]
async fn main() -> std::io::Result<()> {
    let ctx = saoirse::Ctx::new();
    let api = saoirse::api::Api::new(&ctx);
    let addr = ([127, 0, 0, 1], 8080);

    api.serve(addr)
	.await;
    
    Ok(())
}
