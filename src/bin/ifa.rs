// ifascript/src/bin/ifa.rs
// Ògún's Forge: IfáScript CLI — `ifa cast` (v0.2)

use clap::{Parser, Subcommand};
use ifascript::compiler::compile_invocations;
use ifascript::entropy::CowrieOracle;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ifa")]
#[command(version = "0.2.0")]
#[command(about = "IfáScript Ω — Divination as Divine Computation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cast a divination (entropy oracle + optional ritual parsing)
    Cast {
        /// Ritual name to invoke (parses as `invoke <name>;`)
        #[arg(short, long)]
        ritual: Option<String>,

        /// Day for temporal resonance context
        #[arg(short, long)]
        day: Option<String>,

        /// Hermetic gate principle:threshold (e.g. cause_effect:0.95)
        #[arg(short, long)]
        gate: Option<String>,

        /// Witness quorum required
        #[arg(short, long)]
        witness: Option<u8>,
    },

    /// Parse a .ifa source file and emit the AST as JSON
    Build {
        /// Path to .ifa source file
        #[arg(short, long)]
        input: PathBuf,

        /// Output format: json (default) | summary
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Cast {
            ritual,
            day,
            gate,
            witness,
        } => {
            // Use existing entropy oracle
            let mut oracle = CowrieOracle::new("ifa-cast-cli");
            let cast_a = oracle.cast_cowries();
            let cast_b = oracle.cast_cowries();

            println!("🎲 Cast result:");
            println!("   Primary Odù:  {} (0x{:04X})", cast_a, cast_a);
            println!("   Modifier Odù: {} (0x{:04X})", cast_b, cast_b);

            if let Some(d) = &day {
                println!("   Day: {}", d);
            }

            if let Some(name) = ritual {
                // Build a full invoke statement from flags
                let mut stmt = format!("invoke {}", name);
                if let Some(g) = &gate {
                    stmt.push_str(&format!(" with {}", g));
                }
                if let Some(w) = witness {
                    stmt.push_str(&format!(" witness {}", w));
                }
                stmt.push(';');

                match compile_invocations(&stmt) {
                    Ok(invocations) if !invocations.is_empty() => {
                        let inv = &invocations[0];
                        println!("\n   ✓ Ritual: {}", inv.ritual_name);
                        if let Some(gate_spec) = &inv.gate {
                            println!(
                                "   Gate: {:?} threshold={:.2}",
                                gate_spec.principle, gate_spec.threshold
                            );
                        }
                        if let Some(q) = inv.witness_quorum {
                            println!("   Witness quorum: {}", q);
                        }
                        if let Some(s) = &inv.sabbath {
                            println!("   Sabbath: {:?}", s);
                        }
                    }
                    Ok(_) => eprintln!("   ✗ No invocation parsed"),
                    Err(e) => eprintln!("   ✗ Parse error: {}", e),
                }
            }

            Ok(())
        }

        Commands::Build { input, format } => {
            let source = std::fs::read_to_string(&input)
                .map_err(|e| format!("Cannot read {}: {}", input.display(), e))?;

            let invocations = compile_invocations(&source)?;

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&invocations)?);
                }
                "summary" => {
                    println!(
                        "Parsed {} invocation(s) from {}",
                        invocations.len(),
                        input.display()
                    );
                    for (i, inv) in invocations.iter().enumerate() {
                        println!("  [{}] invoke {}", i, inv.ritual_name);
                        if let Some(g) = &inv.gate {
                            println!("       gate: {:?}:{:.2}", g.principle, g.threshold);
                        }
                    }
                }
                _ => eprintln!("Unknown format: {}", format),
            }

            Ok(())
        }
    }
}
