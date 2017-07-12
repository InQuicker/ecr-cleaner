#[macro_use]
extern crate clap;
#[macro_use]
extern crate prettytable;
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

use ecr::{delete_images, list_repositories, list_repository_images};
use error::Error;
use fmt::{images_table, repositories_table};

mod ecr;
mod error;
mod fmt;

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
            SubCommand::with_name("clean")
                .about("Delete images from a repository")
                .arg(
                    Arg::with_name("repository")
                        .help("ECR repository whose images will be deleted")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("count")
                        .help("The number of images to delete if the threshold is met")
                        .required(true)
                        .short("c")
                        .long("count")
                        .takes_value(true)
                        .number_of_values(1),
                )
                .arg(
                    Arg::with_name("threshold")
                        .help(
                            "The number of images that must exist before any will be deleted",
                        )
                        .required(true)
                        .short("t")
                        .long("threshold")
                        .takes_value(true)
                        .number_of_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List ECR repositories or their contents")
                .arg(
                    Arg::with_name("repository")
                        .help("ECR repository to list the contents of")
                        .index(1),
                ),
        )
}

fn main() {
    if let Err(error) = real_main() {
        println!("{}", error);
        exit(1);
    }
}

fn real_main() -> Result<(), Error> {
    let matches = build_cli().get_matches();

    let mut profile_provider = ProfileProvider::new()?;

    if let Some(p) = matches.value_of("profile") {
        profile_provider.set_profile(p);
    }

    let chain_provider = ChainProvider::with_profile_provider(profile_provider);

    let region = matches
        .value_of("region")
        .expect("extracting value of `region`")
        .parse()?;

    let ecr_client = EcrClient::new(default_tls_client()?, chain_provider, region);

    match matches.subcommand() {
        ("list", Some(sub_matches)) => list_subcommand(sub_matches, ecr_client),
        ("clean", Some(sub_matches)) => {
            clean_subcommand(sub_matches, ecr_client)?;

            Ok(())
        }
        _ => Err(Error("unknown subcommand".to_owned())),
    }
}

fn validate_region(region: String) -> Result<(), String> {
    match Region::from_str(region.as_str()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.description().into()),
    }
}

fn clean_subcommand<P, D>(
    arg_matches: &ArgMatches,
    ecr_client: EcrClient<P, D>,
) -> Result<(), Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    let count: u64 = arg_matches
        .value_of("count")
        .expect("accessing `count`")
        .parse()?;
    let threshold: u64 = arg_matches
        .value_of("threshold")
        .expect("accessing `threshold`")
        .parse()?;
    let repository = arg_matches
        .value_of("repository")
        .expect("accessing `repository`");

    let images = list_repository_images(&ecr_client, repository.to_owned())?;

    if images.len() >= threshold as usize {
        println!(
            "Repository {} met threshold of {} images. Deleting the oldest {} images.",
            repository,
            threshold,
            count
        );

        delete_images(&ecr_client, repository, images, count)?;
    }

    Ok(())
}

fn list_subcommand<P, D>(arg_matches: &ArgMatches, ecr_client: EcrClient<P, D>) -> Result<(), Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    if let Some(repo_name) = arg_matches.value_of("repository") {
        let images = list_repository_images(&ecr_client, repo_name.to_string())?;

        let table = images_table(images);

        table.printstd();

        Ok(())
    } else {
        let repositories = list_repositories(&ecr_client)?;

        let table = repositories_table(repositories);

        table.printstd();

        Ok(())
    }
}
