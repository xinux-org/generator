use clap::Parser;
use generator::Args;
use log::error;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args = Args::parse();

    match generator::download(&args.ver, &args.src).await {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            // std::process::exit(1);
        }
    }
}
