use std::cmp::Ordering;

use rusoto_ecr::{ImageDetailList, RepositoryList};

pub fn display_images(mut images: ImageDetailList) {
    println!("Digest\t\tPushed At\t\tSize");

    images.sort_by(|a, b| {
        let a_pushed = a.image_pushed_at.unwrap_or(0f64);
        let b_pushed = b.image_pushed_at.unwrap_or(0f64);

        match a_pushed.partial_cmp(&b_pushed) {
            Some(order) => order.reverse(),
            None => Ordering::Equal,
        }
    });

    for image in images.iter() {
        debug!("Image: {:?}", image);
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

pub fn display_repositories(mut repositories: RepositoryList) {
    println!("Repository Name\t\tURI");
    repositories.sort_by(|a, b| {
        let a_name = match a.repository_name {
            Some(ref n) => n.clone(),
            None => "n/a".to_string(),
        };
        let b_name = match b.repository_name {
            Some(ref n) => n.clone(),
            None => "n/a".to_string(),
        };
        a_name.cmp(&b_name)
    });

    for repository in repositories.iter() {
        debug!("Repository: {:?}", repository);
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
