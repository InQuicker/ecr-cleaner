#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rusoto_core;
extern crate rusoto_ecr;

use std::borrow::Borrow;
use std::cmp::Ordering;
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
use rusoto_ecr::{
    DescribeImagesRequest,
    DescribeRepositoriesRequest,
    Ecr,
    EcrClient,
    ImageDetailList,
    RepositoryList,
};

use error::Error;

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

fn list_repositories<P, D>(ecr_client: EcrClient<P, D>)
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    if let Some(mut repositories) =
        get_repository_list(ecr_client, DescribeRepositoriesRequest::default())
    {
        println!("Repository Name\t\tURI");
        repositories.sort_by(|a, b| {
            let a_name = match a.repository_name.borrow() {
                &Some(ref n) => n.clone(),
                &None => "n/a".to_string(),
            };
            let b_name = match b.repository_name.borrow() {
                &Some(ref n) => n.clone(),
                &None => "n/a".to_string(),
            };
            a_name.cmp(&b_name)
        });

        for repository in repositories.iter() {
            debug!("Repository: {:?}", repository);
            let repository_name = match repository.repository_name.borrow() {
                &Some(ref n) => n.clone(),
                &None => "n/a".to_string(),
            };
            let repository_uri = match repository.repository_uri.borrow() {
                &Some(ref n) => n.clone(),
                &None => "n/a".to_string(),
            };
            println!("{}\t\t{}", repository_name, repository_uri);
        }
    }
}

fn get_repository_list<P, D>(
    ecr_client: EcrClient<P, D>,
    request: DescribeRepositoriesRequest,
) -> Option<RepositoryList>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    match ecr_client.describe_repositories(&request) {
        Ok(response) => {
            debug!("Got a response!");
            debug!("Response {:?}", response);
            let mut repositories = match response.repositories {
                Some(repos) => repos,
                None => return None,
            };

            if let Some(next_token) = response.next_token {
                let new_request = DescribeRepositoriesRequest {
                    next_token: Some(next_token),
                    max_results: request.max_results,
                    registry_id: request.registry_id,
                    repository_names: request.repository_names,
                };
                if let Some(mut more_repos) = get_repository_list(ecr_client, new_request) {
                    repositories.append(&mut more_repos);
                }
            }

            return Some(repositories);
        }
        Err(e) => {
            println!("Could not list repositories: {}", e.description());
            None
        }
    }
}

fn list_repository_images<P, D>(ecr_client: EcrClient<P, D>, repo_name: String)
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    let mut describe_images_request = DescribeImagesRequest::default();
    describe_images_request.repository_name = repo_name;
    if let Some(mut images) = get_repository_image_list(ecr_client, describe_images_request) {
        println!("Digest\t\tPushed At\t\tSize");
        images.sort_by(|a, b| {
            let a_pushed = match a.image_pushed_at {
                Some(pushed) => pushed,
                None => 0f64,
            };
            let b_pushed = match b.image_pushed_at {
                Some(pushed) => pushed,
                None => 0f64,
            };

            match a_pushed.partial_cmp(&b_pushed) {
                Some(order) => order.reverse(),
                None => Ordering::Equal,
            }
        });

        for image in images.iter() {
            debug!("Image: {:?}", image);
            let image_digest = match image.image_digest.borrow() {
                &Some(ref n) => n.clone(),
                &None => "n/a".to_string(),
            };
            println!(
                "{}\t\t{:?}\t{:?}",
                image_digest,
                image.image_pushed_at.unwrap(),
                image.image_size_in_bytes.unwrap()
            );
        }
    }
}

fn get_repository_image_list<P, D>(
    ecr_client: EcrClient<P, D>,
    request: DescribeImagesRequest,
) -> Option<ImageDetailList>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    match ecr_client.describe_images(&request) {
        Ok(response) => {
            debug!("Got a response!");
            debug!("Response {:?}", response);
            let mut images = match response.image_details {
                Some(imgs) => imgs,
                None => return None,
            };

            if let Some(next_token) = response.next_token {
                let new_request = DescribeImagesRequest {
                    filter: request.filter,
                    image_ids: request.image_ids,
                    next_token: Some(next_token),
                    max_results: request.max_results,
                    registry_id: request.registry_id,
                    repository_name: request.repository_name,
                };
                if let Some(mut more_images) = get_repository_image_list(ecr_client, new_request) {
                    images.append(&mut more_images);
                }
            }

            return Some(images);
        }
        Err(e) => {
            println!("Could not list images: {}", e.description());
            None
        }
    }
}
