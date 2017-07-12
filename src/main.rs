#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rusoto_core;
extern crate rusoto_ecr;

use std::error::Error as StdError;
use std::process::exit;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use rusoto_core::{
    ChainProvider,
    DispatchSignedRequest,
    ProfileProvider,
    ProvideAwsCredentials,
    Region,
    default_tls_client,
};
use rusoto_ecr::EcrClient;

use ecr::{list_repositories, list_repository_images};
use error::Error;

mod ecr;
mod error;

fn build_cli() -> App<'static, 'static> {
    app_from_crate!(", ")
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .help("AWS region to operate in")
                .default_value("us-east-1")
                .validator(validate_region),
        )
        .arg(
            Arg::with_name("profile")
                .long("profile")
                .help("AWS credentials profile name")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List ECR repositories or their contents")
                .arg(
                    Arg::with_name("repository").help("ECR repository to list the contents of"),
                ),
        )
}

fn main() {
    if let Err(error) = real_main() {
        error!("{}", error);
        exit(1);
    }
}

fn real_main() -> Result<(), Error> {
    env_logger::init()?;

    let matches = build_cli().get_matches();

    let mut profile_provider = ProfileProvider::new()?;

    if let Some(p) = matches.value_of("profile") {
        debug!("Setting AWS credentials profile to: {}", p);

        profile_provider.set_profile(p);
    }

    let chain_provider = ChainProvider::with_profile_provider(profile_provider);

    let region = matches
        .value_of("region")
        .expect("extracting value of `region`")
        .parse()?;

    debug!("Running with region: {}", region);

    let ecr_client = EcrClient::new(default_tls_client()?, chain_provider, region);

    match matches.subcommand() {
        ("list", Some(sub_m)) => {
            list_subcommand(sub_m, ecr_client);

            Ok(())
        }
        _ => unreachable!(),
    }
}

fn validate_region(region: String) -> Result<(), String> {
    match Region::from_str(region.as_str()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.description().into()),
    }
}

fn list_subcommand<P, D>(arg_matches: &ArgMatches, ecr_client: EcrClient<P, D>)
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    if let Some(repo_name) = arg_matches.value_of("repository") {
        println!("Listing images in {}:", repo_name);
        list_repository_images(ecr_client, repo_name.to_string());
    } else {
        list_repositories(ecr_client);
    }
}
