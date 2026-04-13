mod labwc_ipc;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(
    name = "waysnap",
    about = "Window snapping helper for labwc (Wayland)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install snap keybindings into ~/.config/labwc/rc.xml and reload labwc
    Install {
        /// Modifier key prefix (W=Super, A=Alt, C=Ctrl). Default: W (Super)
        #[arg(long, default_value = "W")]
        modifier: String,
    },
    /// Print the XML snippet to add manually — no file is modified
    ShowConfig {
        /// Modifier key prefix (W=Super, A=Alt, C=Ctrl). Default: W (Super)
        #[arg(long, default_value = "W")]
        modifier: String,
    },
    /// Reload labwc configuration (sends SIGHUP via $LABWC_PID)
    Reload,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Install { modifier } => {
            let snippet = labwc_ipc::keybind_snippet(&modifier);
            match labwc_ipc::install_snippet(&snippet) {
                Ok(path) => {
                    eprintln!("waysnap: keybindings written to {}", path.display());
                    match labwc_ipc::reload_labwc() {
                        Ok(()) => eprintln!("waysnap: labwc reloaded (SIGHUP sent)"),
                        Err(e) => {
                            eprintln!("waysnap: could not reload labwc: {e}");
                            eprintln!("  hint: run `labwc -r` manually, or restart labwc");
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("waysnap: failed to patch rc.xml: {e}");
                    process::exit(1);
                }
            }
        }

        Command::ShowConfig { modifier } => {
            print!("{}", labwc_ipc::keybind_snippet(&modifier));
        }

        Command::Reload => match labwc_ipc::reload_labwc() {
            Ok(()) => eprintln!("waysnap: labwc reloaded"),
            Err(e) => {
                eprintln!("waysnap: {e}");
                process::exit(1);
            }
        },
    }
}
