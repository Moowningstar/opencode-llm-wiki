use clap::{Parser, Subcommand};
use std::sync::Arc;
use llm_wiki_server::storage::VectorStorage;

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
    
    Migrate {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(short, long)]
        project: String,
    },
    
    Check {
        #[arg(short, long)]
        project: Option<String>,
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
            
            #[cfg(feature = "ruvector")]
            let storage = Arc::new(llm_wiki_server::storage::RuVectorStorage::new(project_path.to_string(), 2048).await?);
            
            #[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
            let storage = Arc::new(llm_wiki_server::storage::LanceDbStorage::new(project_path));
            
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
            
            #[cfg(feature = "ruvector")]
            let storage = Arc::new(llm_wiki_server::storage::RuVectorStorage::new(project_path.to_string(), 2048).await?);
            
            #[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
            let storage = Arc::new(llm_wiki_server::storage::LanceDbStorage::new(project_path));
            
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
        Commands::Migrate { from, to, project } => {
            println!("🔄 Migrating from {} to {}", from, to);
            println!("   Project: {}", project);
            
            if from == "lancedb" && to == "ruvector" {
                #[cfg(all(feature = "ruvector", feature = "lancedb-backend"))]
                {
                    println!("⚠️  Migration not yet implemented");
                    println!("   This will be implemented in Phase 1.7");
                }
                
                #[cfg(not(all(feature = "ruvector", feature = "lancedb-backend")))]
                {
                    eprintln!("Error: Both 'ruvector' and 'lancedb-backend' features must be enabled for migration");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Unsupported migration path: {} -> {}", from, to);
                std::process::exit(1);
            }
        }
        Commands::Check { project } => {
            let project_path = project.unwrap_or_else(|| ".".to_string());
            println!("🔍 Checking storage backend...");
            println!("   Project: {}", project_path);
            
            #[cfg(feature = "ruvector")]
            {
                println!("   Backend: RuVector");
                let storage = llm_wiki_server::storage::ruvector_impl::RuVectorStorage::new(
                    project_path.clone(), 
                    2048
                ).await?;
                let count = storage.count().await?;
                println!("   ✅ RuVector is working");
                println!("   Total chunks: {}", count);
            }
            
            #[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
            {
                println!("   Backend: LanceDB");
                let storage = llm_wiki_server::storage::LanceDbStorage::new(project_path);
                let count = storage.count().await?;
                println!("   ✅ LanceDB is working");
                println!("   Total chunks: {}", count);
            }
        }
    }
    
    Ok(())
}
