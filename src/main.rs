use clap::{Parser, Subcommand};

use protohackers::{means_to_an_end, prime_time, smoke_test};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long, value_parser)]
    port: Option<usize>,
}

#[derive(Subcommand)]
enum Commands {
    SmokeTest {
        client_or_server: String,
        client_string: Option<String>,
        client_destination_url: Option<String>,
    },
    PrimeTime,
    MeansToAnEnd,
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
                protohackers::run_server(args.port, 5, smoke_test::handle_connection);
            }
            ("client", Some(client_string)) => {
                smoke_test::run_client(client_destination_url, client_string.as_bytes())
            }
            ("client", None) => smoke_test::run_client(client_destination_url, b"Hello world!"),
            _ => panic!("Invalid smoketest argument '{}'.", client_or_server),
        },
        Commands::PrimeTime => {
            protohackers::run_server(args.port, 5, prime_time::handle_connection)
        }
        Commands::MeansToAnEnd => {
            protohackers::run_server(args.port, 5, means_to_an_end::handle_connection)
        }
    }
}
