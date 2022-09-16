use clap::{Parser, Subcommand};

mod smoke_test;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SmokeTest {
        client_or_server: String,
        client_string: Option<String>,
    },
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    match args.command {
        Commands::SmokeTest {
            client_or_server,
            client_string,
        } => match (client_or_server.as_str(), client_string) {
            ("server", _) => smoke_test::run_server(),
            ("client", Some(client_string)) => smoke_test::run_client(client_string.as_bytes()),
            ("client", None) => smoke_test::run_client(b"Hello world!"),
            _ => panic!("Invalid smoketest argument '{}'.", client_or_server),
        },
    }
}
