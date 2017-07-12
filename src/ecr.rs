use std::borrow::Borrow;
use std::cmp::Ordering;
use std::error::Error as StdError;

use rusoto_core::{DispatchSignedRequest, ProvideAwsCredentials};
use rusoto_ecr::{
    DescribeImagesRequest,
    DescribeRepositoriesRequest,
    Ecr,
    EcrClient,
    ImageDetailList,
    RepositoryList,
};

pub fn get_repository_image_list<P, D>(
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
            error!("Could not list images: {}", e.description());
            None
        }
    }
}

pub fn get_repository_list<P, D>(
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
            error!("Could not list repositories: {}", e.description());
            None
        }
    }
}

pub fn list_repositories<P, D>(ecr_client: EcrClient<P, D>)
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


pub fn list_repository_images<P, D>(ecr_client: EcrClient<P, D>, repo_name: String)
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
