use clap::Parser;
use ifascript::cosmogram::{CosmogramEngine, Day};
use rand::RngCore;

#[derive(Parser, Debug)]
#[command(name = "cast_state", about = "Cast a Cosmogram State")]
struct Args {
    /// Tier level (1-7)
    #[arg(short, long, default_value_t = 1)]
    tier: u8,

    /// Odu ID
    #[arg(short, long, default_value_t = 1)]
    odu_id: u16,

    /// Day of the week
    #[arg(short, long, default_value = "sunday")]
    day: Day,

    /// Unix timestamp (defaults to current time)
    #[arg(long)]
    timestamp: Option<i64>,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Run validated cast (with hermetic gate check)
    #[arg(long)]
    validated: bool,
}

fn main() {
    let args = Args::parse();

    // Load entropy from env or generate fresh
    let entropy: Vec<u8> = match std::env::var("OSOVM_SEED") {
        Ok(hex_str) => hex::decode(hex_str.trim()).unwrap_or_else(|_| {
            eprintln!("Warning: OSOVM_SEED is not valid hex, generating fresh entropy");
            fresh_entropy()
        }),
        Err(_) => fresh_entropy(),
    };

    let engine = CosmogramEngine::from_entropy(entropy);
    let timestamp = args.timestamp.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    });

    if args.validated {
        match engine.cast_validated(args.tier, args.odu_id, args.day, timestamp) {
            Ok(validated) => {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&validated).unwrap());
                } else {
                    println!("ValidatedCast:");
                    println!("  gates_passed: {}", validated.gates_passed);
                    println!("  violation_count: {}", validated.violation_count);
                    println!("  odu_id: {}", validated.state.odu_id);
                    println!("  tier: {}", validated.state.tier);
                    println!("  day: {}", validated.state.day);
                    println!("  window_open: {}", validated.state.window_open);
                    println!("  entropy_hash: {}", validated.state.entropy_hash);
                    println!("  timestamp: {}", validated.timestamp);
                }
            }
            Err(e) => {
                eprintln!("Cast error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match engine.cast(args.tier, args.odu_id, args.day, timestamp) {
            Ok(state) => {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&state).unwrap());
                } else {
                    println!("CosmogramState:");
                    println!("  odu_id: {}", state.odu_id);
                    println!("  tier: {}", state.tier);
                    println!("  day: {}", state.day);
                    println!("  access_class: {:?}", state.access_class);
                    println!("  memory_tier: {:?}", state.memory_tier);
                    println!("  window_open: {}", state.window_open);
                    println!("  entropy_hash: {}", state.entropy_hash);
                    println!("  timestamp: {}", state.timestamp);
                }
            }
            Err(e) => {
                eprintln!("Cast error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn fresh_entropy() -> Vec<u8> {
    let mut entropy = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut entropy);
    entropy
}
