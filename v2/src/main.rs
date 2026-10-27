#[tokio::main(worker_threads = 2)]
async fn main() {
    if let Err(error) =
        tezbar_extension_sdk::serve(tezbar_convert::adapter::Converter::default()).await
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
