// ifascript/src/bin/ifa.rs
// Ògún's Forge: IfáScript CLI — `ifa cast` (v0.2)

use clap::{Parser, Subcommand};
use ifascript::compiler::compile_invocations;
use ifascript::IfaVM;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ifa")]
#[command(version = "0.2.0")]
#[command(about = "IfáScript Ω — Divination as Divine Computation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Which corpus a cast should resolve against — the agent-native Digital
/// Calabash, the traditional Òdù Ifá, or both from the same cowrie throw.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Corpus {
    Agent,
    Human,
    Both,
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

        /// Which corpus/corpora to resolve the cast against
        #[arg(short = 'c', long, value_enum, default_value_t = Corpus::Both)]
        corpus: Corpus,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
            corpus,
            json,
        } => {
            let mut vm = IfaVM::with_intent("ifa-cast-cli");

            if json {
                let payload = match corpus {
                    Corpus::Agent => {
                        let odu = vm.cast_odu_full();
                        serde_json::json!({ "agent": odu_json(odu) })
                    }
                    Corpus::Human => {
                        let odu = vm.cast_dual_full().1;
                        serde_json::json!({ "human": odu_json(odu) })
                    }
                    Corpus::Both => {
                        let (agent, human) = vm.cast_dual_full();
                        serde_json::json!({ "agent": odu_json(agent), "human": odu_json(human) })
                    }
                };
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("🎲 Cast result:");
                match corpus {
                    Corpus::Agent => print_odu("Agent (Digital Calabash)", vm.cast_odu_full()),
                    Corpus::Human => print_odu("Human (Òdù Ifá)", vm.cast_dual_full().1),
                    Corpus::Both => {
                        let (agent, human) = vm.cast_dual_full();
                        print_odu("Agent (Digital Calabash)", agent);
                        print_odu("Human (Òdù Ifá)", human);
                    }
                }
            }

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

fn print_odu(label: &str, odu: &ifascript::Odu) {
    println!("\n   {label}");
    println!("     Index:      {}", odu.index);
    println!("     Name:       {}", odu.name);
    println!("     Universal:  {}", odu.universal_name);
    println!("     Archetypes: {:?}", odu.archetypes);
    println!("     Prescriptions:");
    for p in odu.prescriptions {
        println!("       - {p}");
    }
}

fn odu_json(odu: &ifascript::Odu) -> serde_json::Value {
    serde_json::json!({
        "index": odu.index,
        "name": odu.name,
        "universal_name": odu.universal_name,
        "archetypes": odu.archetypes,
        "prescriptions": odu.prescriptions,
    })
}
