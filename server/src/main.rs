use std::net::SocketAddr;
use std::path::PathBuf;
use thermite_server::game_server::GameServer;
use thermite_server::game_state::MatchConfig;
use thermite_server::map_template::MapTemplate;
use uuid::Uuid;

#[derive(Debug)]
struct Args {
    match_id: Uuid,
    port: u16,
    map_path: PathBuf,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() != 4 {
            return Err(format!(
                "Usage: {} <match_id> <port> <map_path>",
                args.first().map(|s| s.as_str()).unwrap_or("game-server")
            ));
        }

        let match_id = Uuid::parse_str(&args[1])
            .map_err(|e| format!("Invalid match_id UUID: {}", e))?;

        let port = args[2]
            .parse::<u16>()
            .map_err(|e| format!("Invalid port number: {}", e))?;

        let map_path = PathBuf::from(&args[3]);

        Ok(Args {
            match_id,
            port,
            map_path,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command-line arguments
    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!(
        "Starting Game Server for match {} on port {}",
        args.match_id,
        args.port
    );

    // Load map template and generate grid
    tracing::info!("Loading map template from {:?}", args.map_path);
    let template = MapTemplate::load_from_file(&args.map_path)?;
    let grid = template.generate_grid(None)?;
    tracing::info!("Map loaded: {}x{}", grid.width, grid.height);

    // Create game server with config from map template
    let config = MatchConfig {
        duration_ms: template.raid_duration_seconds * 1000,
        tick_rate_ms: 50,            // 20Hz
        lobby_duration_ms: 5 * 1000, // 5 seconds lobby
    };
    tracing::info!("Match duration: {}s", template.raid_duration_seconds);
    let (server, command_rx) = GameServer::new(args.match_id, grid, config);

    // Bind WebSocket server address
    let addr: SocketAddr = ([0, 0, 0, 0], args.port).into();

    tracing::info!("Game Server initialized, starting services...");

    // Spawn WebSocket server task
    let ws_server = server.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_server.run_websocket_server(addr).await {
            tracing::error!("WebSocket server error: {}", e);
        }
    });

    // Spawn tick loop task
    let tick_server = server.clone();
    let tick_handle = tokio::spawn(async move {
        tick_server.run_tick_loop(command_rx).await;
    });

    tracing::info!("Game Server running - WebSocket: {}, Tick: 20Hz", addr);

    // Wait for either task to complete (shouldn't happen in normal operation)
    tokio::select! {
        _ = ws_handle => {
            tracing::warn!("WebSocket server task completed");
        }
        _ = tick_handle => {
            tracing::warn!("Tick loop task completed");
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
        }
    };

    tracing::info!("Game Server shutting down");
    Ok(())
}
