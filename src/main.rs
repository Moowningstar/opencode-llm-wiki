//! OpenCode LLM Wiki - API 服务器

use clap::Parser;
use llm_wiki_server::api::start_api_server;

#[derive(Parser)]
#[command(name = "llm-wiki-server")]
#[command(about = "OpenCode LLM Wiki API Server")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
    
    #[arg(short, long, default_value = "19828")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🚀 OpenCode LLM Wiki Server");
    println!("   Version: 0.1.0");
    println!("   Host: {}", args.host);
    println!("   Port: {}", args.port);
    println!();
    
    start_api_server(args.port).await?;
    
    Ok(())
}
