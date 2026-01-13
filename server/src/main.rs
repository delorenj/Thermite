use chrono;
use std::net::SocketAddr;
use std::path::PathBuf;
use thermite_server::game_server::GameServer;
use thermite_server::game_state::MatchConfig;
use thermite_server::map_template::MapTemplate;
use thermite_server::rabbitmq::{MatchEvent, RabbitMQPublisher};
use uuid::Uuid;

#[derive(Debug)]
struct Args {
    match_id: Uuid,
    port: u16,
    map_path: PathBuf,
    rabbitmq_url: Option<String>,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 4 || args.len() > 5 {
            return Err(format!(
                "Usage: {} <match_id> <port> <map_path> [rabbitmq_url]",
                args.first().map(|s| s.as_str()).unwrap_or("game-server")
            ));
        }

        let match_id = Uuid::parse_str(&args[1])
            .map_err(|e| format!("Invalid match_id UUID: {}", e))?;

        let port = args[2]
            .parse::<u16>()
            .map_err(|e| format!("Invalid port number: {}", e))?;

        let map_path = PathBuf::from(&args[3]);

        let rabbitmq_url = args.get(4).cloned();

        Ok(Args {
            match_id,
            port,
            map_path,
            rabbitmq_url,
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

    // Connect to RabbitMQ if URL provided (before creating game server)
    let rabbitmq = if let Some(ref url) = args.rabbitmq_url {
        match RabbitMQPublisher::connect(url, "match_events").await {
            Ok(publisher) => {
                tracing::info!("Connected to RabbitMQ");
                Some(std::sync::Arc::new(publisher))
            }
            Err(e) => {
                tracing::warn!("Failed to connect to RabbitMQ: {}. Continuing without events.", e);
                None
            }
        }
    } else {
        tracing::info!("No RabbitMQ URL provided, running without event publishing");
        None
    };

    // Create game server with config from map template
    let config = MatchConfig {
        duration_ms: template.raid_duration_seconds * 1000,
        tick_rate_ms: 50,            // 20Hz
        lobby_duration_ms: 5 * 1000, // 5 seconds lobby
    };
    tracing::info!("Match duration: {}s", template.raid_duration_seconds);
    let (server, command_rx) = GameServer::new(args.match_id, grid, config, rabbitmq.clone());

    // Bind WebSocket server address
    let addr: SocketAddr = ([0, 0, 0, 0], args.port).into();

    tracing::info!("Game Server initialized, starting services...");

    // Emit match started event
    if let Some(ref rmq) = rabbitmq {
        let event = MatchEvent::MatchStarted {
            match_id: args.match_id,
            player_count: 0, // Will be updated as players connect
            map_name: template.name.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        if let Err(e) = rmq.publish_event(&event).await {
            tracing::warn!("Failed to publish match started event: {}", e);
        } else {
            tracing::info!("Published match started event");
        }
    }

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
    let shutdown_reason = tokio::select! {
        _ = ws_handle => {
            tracing::warn!("WebSocket server task completed");
            "websocket_completed"
        }
        _ = tick_handle => {
            tracing::warn!("Tick loop task completed");
            "tick_completed"
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
            "signal"
        }
    };

    // Emit match ended event
    if let Some(ref rmq) = rabbitmq {
        let event = MatchEvent::MatchEnded {
            match_id: args.match_id,
            duration_ms: 0, // TODO: Track actual duration
            survivors: vec![], // TODO: Get from game state
            timestamp: chrono::Utc::now().timestamp(),
        };
        if let Err(e) = rmq.publish_event(&event).await {
            tracing::warn!("Failed to publish match ended event: {}", e);
        } else {
            tracing::info!("Published match ended event (reason: {})", shutdown_reason);
        }
    }

    tracing::info!("Game Server shutting down");
    Ok(())
}
