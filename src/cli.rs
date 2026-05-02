use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "llm-wiki")]
#[command(about = "OpenCode LLM Wiki CLI", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        
        #[arg(short, long, default_value = "19828")]
        port: u16,
    },
    
    Init {
        path: String,
        #[arg(short, long, default_value = "general")]
        template: String,
    },
    
    Ingest {
        file: String,
        #[arg(short, long)]
        project: Option<String>,
    },
    
    Query {
        query: String,
        #[arg(short, long)]
        project: Option<String>,
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { host, port } => {
            println!("🚀 Starting LLM Wiki Server on {}:{}", host, port);
            llm_wiki_server::api::server::start_api_server(port).await?;
        }
        Commands::Init { path, template } => {
            println!("📁 Initializing wiki at: {}", path);
            println!("   Template: {}", template);
            std::fs::create_dir_all(&path)?;
            println!("✅ Wiki initialized successfully");
        }
        Commands::Ingest { file, project } => {
            let project_path = project.unwrap_or_else(|| ".".to_string());
            println!("📥 Ingesting file: {}", file);
            println!("   Project: {}", project_path);
            
            let content = tokio::fs::read_to_string(&file).await?;
            
            let api_key = std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "sk-placeholder".to_string());
            
            let storage = Arc::new(llm_wiki_server::storage::lancedb_impl::LanceDbStorage::new(project_path));
            let embedding_service = llm_wiki_server::services::embedding::EmbeddingService::new(
                llm_wiki_server::services::embedding::EmbeddingConfig {
                    api_key,
                    ..Default::default()
                }
            )?;
            let ingest_service = llm_wiki_server::services::ingest::IngestService::new(
                storage.clone(),
                embedding_service.clone(),
                llm_wiki_server::services::ingest::IngestConfig::default(),
            )?;
            
            let result = ingest_service.ingest_file_blocks(&content).await?;
            println!("✅ File ingested successfully");
            println!("   Pages processed: {}", result.pages_processed);
            println!("   Chunks created: {}", result.chunks_created);
        }
        Commands::Query { query, project, limit } => {
            let project_path = project.unwrap_or_else(|| ".".to_string());
            println!("🔍 Querying: {}", query);
            println!("   Project: {}", project_path);
            
            let api_key = std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "sk-placeholder".to_string());
            
            let storage = Arc::new(llm_wiki_server::storage::lancedb_impl::LanceDbStorage::new(project_path));
            let embedding_service = llm_wiki_server::services::embedding::EmbeddingService::new(
                llm_wiki_server::services::embedding::EmbeddingConfig {
                    api_key,
                    ..Default::default()
                }
            )?;
            let query_service = llm_wiki_server::services::query::QueryService::new(
                storage.clone(),
                embedding_service.clone(),
                llm_wiki_server::services::query::QueryConfig {
                    top_k: limit,
                    ..Default::default()
                },
            );
            
            let result = query_service.query(&query).await?;
            
            println!("\n📊 Found {} results:", result.results.len());
            for (i, res) in result.results.iter().enumerate() {
                println!("\n{}. [Score: {:.4}] {}", i + 1, res.score, res.page_id);
                println!("   {}", res.chunk_text.chars().take(200).collect::<String>());
                if res.chunk_text.len() > 200 {
                    println!("   ...");
                }
            }
        }
    }
    
    Ok(())
}
