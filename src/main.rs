use clap::{Parser, Subcommand};

mod prime_time;
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
        client_destination_url: Option<String>,
    },
    PrimeTime,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    match args.command {
        Commands::SmokeTest {
            client_or_server,
            client_string,
            client_destination_url,
        } => match (client_or_server.as_str(), client_string) {
            ("server", _) => {
                protohackers::run_server(5, smoke_test::handle_connection);
            }
            ("client", Some(client_string)) => {
                smoke_test::run_client(client_destination_url, client_string.as_bytes())
            }
            ("client", None) => smoke_test::run_client(client_destination_url, b"Hello world!"),
            _ => panic!("Invalid smoketest argument '{}'.", client_or_server),
        },
        Commands::PrimeTime => protohackers::run_server(5, prime_time::handle_connection),
    }
}
