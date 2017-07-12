use rusoto_ecr::{ImageDetailList, RepositoryList};

pub fn display_images(images: ImageDetailList) {
    println!("Digest\t\tPushed At\t\tSize");

    for image in images.iter() {
        let image_digest = match image.image_digest {
            Some(ref n) => n.clone(),
            None => "n/a".to_string(),
        };
        println!(
            "{}\t\t{:?}\t{:?}",
            image_digest,
            image.image_pushed_at.unwrap(),
            image.image_size_in_bytes.unwrap()
        );
    }
}

pub fn display_repositories(repositories: RepositoryList) {
    println!("Repository Name\t\tURI");

    for repository in repositories.iter() {
        let repository_name = match repository.repository_name {
            Some(ref n) => n.clone(),
            None => "n/a".to_string(),
        };
        let repository_uri = match repository.repository_uri {
            Some(ref n) => n.clone(),
            None => "n/a".to_string(),
        };
        println!("{}\t\t{}", repository_name, repository_uri);
    }
}
