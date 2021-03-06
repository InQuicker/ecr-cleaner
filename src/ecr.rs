use std::cmp::Ordering;
use std::error::Error as StdError;

use rusoto_core::{DispatchSignedRequest, ProvideAwsCredentials};
use rusoto_ecr::{
    BatchDeleteImageRequest,
    DescribeImagesRequest,
    DescribeRepositoriesRequest,
    Ecr,
    EcrClient,
    ImageDetailList,
    ImageIdentifier,
    RepositoryList,
};

use error::Error;

pub fn delete_images<P, D>(
    ecr_client: &EcrClient<P, D>,
    repository_name: &str,
    images: ImageDetailList,
    count: u64,
) -> Result<(), Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    let image_identifiers: Vec<ImageIdentifier> = images
        .into_iter()
        .take(count as usize)
        .map(|image| {
            ImageIdentifier {
                image_digest: image.image_digest,
                image_tag: None,
            }
        })
        .collect();

    let request = BatchDeleteImageRequest {
        image_ids: image_identifiers,
        registry_id: None,
        repository_name: repository_name.to_owned(),
    };

    ecr_client.batch_delete_image(&request)?;

    Ok(())
}

pub fn get_repository_image_list<P, D>(
    ecr_client: &EcrClient<P, D>,
    request: DescribeImagesRequest,
) -> Result<ImageDetailList, Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    match ecr_client.describe_images(&request) {
        Ok(response) => {
            let mut images = match response.image_details {
                Some(images) => images,
                None => return Err(Error("no images found in repository".to_owned())),
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

                let mut more_images = get_repository_image_list(ecr_client, new_request)?;

                images.append(&mut more_images);
            }

            return Ok(images);
        }
        Err(error) => Err(Error(
            format!("Could not list images: {}", error.description()),
        )),
    }
}

pub fn get_repository_list<P, D>(
    ecr_client: &EcrClient<P, D>,
    request: DescribeRepositoriesRequest,
) -> Result<RepositoryList, Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    match ecr_client.describe_repositories(&request) {
        Ok(response) => {
            let mut repositories = match response.repositories {
                Some(repositories) => repositories,
                None => return Err(Error("no repositories found".to_owned())),
            };

            if let Some(next_token) = response.next_token {
                let new_request = DescribeRepositoriesRequest {
                    next_token: Some(next_token),
                    max_results: request.max_results,
                    registry_id: request.registry_id,
                    repository_names: request.repository_names,
                };

                let mut more_repos = get_repository_list(ecr_client, new_request)?;

                repositories.append(&mut more_repos);
            }

            return Ok(repositories);
        }
        Err(e) => Err(Error(
            format!("Could not list repositories: {}", e.description()),
        )),
    }
}

pub fn list_repositories<P, D>(ecr_client: &EcrClient<P, D>) -> Result<RepositoryList, Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    let mut repositories = get_repository_list(ecr_client, DescribeRepositoriesRequest::default())?;

    repositories.sort_by(|a, b| a.repository_name.cmp(&b.repository_name));

    Ok(repositories)
}


pub fn list_repository_images<P, D>(
    ecr_client: &EcrClient<P, D>,
    repo_name: String,
) -> Result<ImageDetailList, Error>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    let mut describe_images_request = DescribeImagesRequest::default();

    describe_images_request.repository_name = repo_name;

    let mut images = get_repository_image_list(ecr_client, describe_images_request)?;

    images.sort_by(|a, b| {
        a.image_pushed_at
            .partial_cmp(&b.image_pushed_at)
            .unwrap_or(Ordering::Equal)
    });

    Ok(images)
}
