use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::path::Path;
use llm_wiki_server::storage::VectorStorage;

#[derive(Parser)]
#[command(name = "llm-wiki")]
#[command(about = "OpenCode LLM Wiki CLI", long_about = None)]
#[command(version = "1.1.1")]
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
    
    Projects {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    
    MigrateToGlobal {
        #[arg(long, help = "Scan directory for old projects (default: current directory)")]
        scan_dir: Option<String>,
        #[arg(long, help = "Global storage root (default: ~/.opencode-llm-wiki)")]
        global_root: Option<String>,
        #[arg(long, help = "Dry run - show what would be migrated without actually migrating")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    List {
        #[arg(long)]
        global_root: Option<String>,
    },
    
    Stats {
        #[arg(long)]
        global_root: Option<String>,
        #[arg(short, long)]
        project: Option<String>,
    },
    
    Cleanup {
        #[arg(long)]
        global_root: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    
    Verify {
        #[arg(long)]
        global_root: Option<String>,
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
                project_path.clone(),
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
                let count = storage.count(None).await?;
                println!("   ✅ RuVector is working");
                println!("   Total chunks: {}", count);
            }
            
            #[cfg(all(feature = "lancedb-backend", not(feature = "ruvector")))]
            {
                println!("   Backend: LanceDB");
                let storage = llm_wiki_server::storage::LanceDbStorage::new(project_path);
                let count = storage.count(None).await?;
                println!("   ✅ LanceDB is working");
                println!("   Total chunks: {}", count);
            }
        }
        
        Commands::Projects { command } => {
            use llm_wiki_server::types::storage::GlobalStoragePaths;
            use llm_wiki_server::storage::{ProjectRegistry, VectorDeduplicator};
            
            match command {
                ProjectCommands::List { global_root } => {
                    let paths = if let Some(root) = global_root {
                        GlobalStoragePaths::new(root)
                    } else {
                        GlobalStoragePaths::default()
                    };
                    
                    let registry = ProjectRegistry::new(Path::new(&paths.root))?;
                    let projects = registry.list_projects()?;
                    
                    if projects.is_empty() {
                        println!("📋 No projects registered yet");
                    } else {
                        println!("📋 Registered Projects ({}):", projects.len());
                        println!();
                        for (idx, project) in projects.iter().enumerate() {
                            println!("  {}. {}", idx + 1, project.name);
                            println!("     Path: {}", project.path);
                            println!("     Created: {}", project.created_at);
                            println!("     Last accessed: {}", project.last_accessed);
                            println!("     Pages: {}, Chunks: {}, Disk: {} bytes", 
                                project.stats.page_count, project.stats.chunk_count, project.stats.disk_usage);
                            println!();
                        }
                    }
                }
                
                ProjectCommands::Stats { global_root, project } => {
                    let paths = if let Some(root) = global_root {
                        GlobalStoragePaths::new(root)
                    } else {
                        GlobalStoragePaths::default()
                    };
                    
                    let registry = ProjectRegistry::new(Path::new(&paths.root))?;
                    
                    if let Some(project_id) = project {
                        if let Some(info) = registry.get_project(&project_id)? {
                            println!("📊 Project Statistics: {}", info.name);
                            println!("   Path: {}", info.path);
                            println!("   Pages: {}", info.stats.page_count);
                            println!("   Chunks: {}", info.stats.chunk_count);
                            println!("   Disk usage: {} bytes", info.stats.disk_usage);
                        } else {
                            eprintln!("❌ Project not found: {}", project_id);
                            std::process::exit(1);
                        }
                    } else {
                        let global_stats = registry.get_global_stats()?;
                        println!("📊 Global Statistics:");
                        println!("   Total projects: {}", global_stats.total_projects);
                        println!("   Total pages: {}", global_stats.total_pages);
                        println!("   Total chunks: {}", global_stats.total_chunks);
                        println!("   Total disk usage: {} bytes", global_stats.total_disk_usage);
                        
                        let dedup = VectorDeduplicator::new(Path::new(&paths.root))?;
                        let dedup_stats = dedup.get_stats()?;
                        println!();
                        println!("🔄 Deduplication Statistics:");
                        println!("   Total chunks: {}", dedup_stats.total_chunks);
                        println!("   Unique vectors: {}", dedup_stats.unique_vectors);
                        println!("   Reused vectors: {}", dedup_stats.reused_vectors);
                        println!("   Reuse rate: {:.2}%", dedup_stats.reuse_rate * 100.0);
                    }
                }
                
                ProjectCommands::Cleanup { global_root, dry_run } => {
                    let paths = if let Some(root) = global_root {
                        GlobalStoragePaths::new(root)
                    } else {
                        GlobalStoragePaths::default()
                    };
                    
                    println!("🧹 Cleaning up unused vectors...");
                    if dry_run {
                        println!("   (Dry run mode - no changes will be made)");
                    }
                    
                    let dedup = VectorDeduplicator::new(Path::new(&paths.root))?;
                    let integrity = dedup.verify_integrity()?;
                    
                    if integrity.orphaned_refs.is_empty() && !integrity.has_mismatch {
                        println!("✅ No cleanup needed - all data is consistent");
                    } else {
                        if !integrity.orphaned_refs.is_empty() {
                            println!("   Found {} orphaned references", integrity.orphaned_refs.len());
                        }
                        if integrity.has_mismatch {
                            println!("   Found count mismatch: hash_index={}, ref_counter={}", 
                                integrity.hash_index_count, integrity.ref_counter_count);
                        }
                        
                        if !dry_run {
                            println!("   ⚠️  Automatic cleanup not yet implemented");
                            println!("   Please run with --dry-run to see issues first");
                        }
                    }
                }
                
                ProjectCommands::Verify { global_root } => {
                    let paths = if let Some(root) = global_root {
                        GlobalStoragePaths::new(root)
                    } else {
                        GlobalStoragePaths::default()
                    };
                    
                    println!("🔍 Verifying data integrity...");
                    
                    let dedup = VectorDeduplicator::new(Path::new(&paths.root))?;
                    let integrity = dedup.verify_integrity()?;
                    
                    if integrity.orphaned_refs.is_empty() && !integrity.has_mismatch {
                        println!("✅ All data is consistent");
                    } else {
                        println!("❌ Found integrity issues:");
                        
                        if !integrity.orphaned_refs.is_empty() {
                            println!();
                            println!("   Orphaned references ({}):", integrity.orphaned_refs.len());
                            for hash in integrity.orphaned_refs.iter().take(10) {
                                println!("     • {}", hash);
                            }
                            if integrity.orphaned_refs.len() > 10 {
                                println!("     ... and {} more", integrity.orphaned_refs.len() - 10);
                            }
                        }
                        
                        if integrity.has_mismatch {
                            println!();
                            println!("   Count mismatch:");
                            println!("     Hash index count: {}", integrity.hash_index_count);
                            println!("     Ref counter count: {}", integrity.ref_counter_count);
                        }
                        
                        println!();
                        println!("   Run 'projects cleanup' to fix these issues");
                    }
                }
            }
        }
        
        Commands::MigrateToGlobal { scan_dir, global_root, dry_run } => {
            use llm_wiki_server::types::storage::GlobalStoragePaths;
            use llm_wiki_server::storage::{DataMigrator, MigrationStats};
            use std::path::PathBuf;
            
            let paths = if let Some(root) = global_root {
                GlobalStoragePaths::new(root)
            } else {
                GlobalStoragePaths::default()
            };
            
            let scan_path = scan_dir.unwrap_or_else(|| ".".to_string());
            
            println!("🔄 Migrating projects to global storage");
            println!("   Scan directory: {}", scan_path);
            println!("   Global root: {}", paths.root);
            if dry_run {
                println!("   Mode: DRY RUN (no changes will be made)");
            }
            println!();
            
            let mut migration_tool = DataMigrator::new(Some(PathBuf::from(&paths.root)))?;
            
            println!("🔍 Scanning for old projects...");
            let old_projects = migration_tool.scan_for_old_projects(Path::new(&scan_path))?;
            
            if old_projects.is_empty() {
                println!("✅ No old projects found to migrate");
                return Ok(());
            }
            
            println!("   Found {} project(s) to migrate:", old_projects.len());
            for (idx, proj_path) in old_projects.iter().enumerate() {
                println!("     {}. {}", idx + 1, proj_path.display());
            }
            println!();
            
            if dry_run {
                println!("✅ Dry run complete - no changes made");
                println!("   Run without --dry-run to perform the migration");
                return Ok(());
            }
            
            println!("🚀 Starting migration...");
            let mut total_stats = MigrationStats {
                projects_found: old_projects.len(),
                projects_migrated: 0,
                total_chunks: 0,
                unique_vectors: 0,
                reused_vectors: 0,
                bytes_saved: 0,
            };
            
            let mut total_failed = 0;
            for (idx, proj_path) in old_projects.iter().enumerate() {
                println!();
                println!("  [{}/{}] Migrating: {}", idx + 1, old_projects.len(), proj_path.display());
                
                match migration_tool.migrate_project(proj_path, false) {
                    Ok(stats) => {
                        println!("     ✅ Success");
                        println!("        Chunks: {}", stats.total_chunks);
                        println!("        Unique vectors: {}", stats.unique_vectors);
                        println!("        Reused vectors: {}", stats.reused_vectors);
                        total_stats.projects_migrated += stats.projects_migrated;
                        total_stats.total_chunks += stats.total_chunks;
                        total_stats.unique_vectors += stats.unique_vectors;
                        total_stats.reused_vectors += stats.reused_vectors;
                        total_stats.bytes_saved += stats.bytes_saved;
                    }
                    Err(e) => {
                        println!("     ❌ Failed: {}", e);
                        total_failed += 1;
                    }
                }
            }
            
            println!();
            println!("📊 Migration Summary:");
            println!("   Projects migrated: {}/{}", total_stats.projects_migrated, total_stats.projects_found);
            println!("   Total chunks: {}", total_stats.total_chunks);
            println!("   Unique vectors: {}", total_stats.unique_vectors);
            println!("   Reused vectors: {}", total_stats.reused_vectors);
            if total_stats.unique_vectors + total_stats.reused_vectors > 0 {
                let reuse_rate = (total_stats.reused_vectors as f64) / ((total_stats.unique_vectors + total_stats.reused_vectors) as f64) * 100.0;
                println!("   Deduplication rate: {:.2}%", reuse_rate);
            }
            println!("   Disk space saved: {} bytes", total_stats.bytes_saved);
            
            if total_failed == 0 {
                println!();
                println!("✅ All projects migrated successfully!");
                println!("   Old project data can be safely deleted after verification");
            }
        }
    }
    
    Ok(())
}
