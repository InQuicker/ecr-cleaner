use prettytable::Table;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};
use rusoto_ecr::{ImageDetailList, RepositoryList};

pub fn images_table(images: ImageDetailList) -> Table {
    let mut table = Table::new();

    table.set_format(format());
    table.set_titles(row!["Digest", "Pushed at", "Size"]);

    for image in images {
        table.add_row(row![
            image.image_digest.unwrap_or("".to_owned()),
            image.image_pushed_at.unwrap_or(0f64),
            image.image_size_in_bytes.unwrap_or(0)
        ]);
    }

    table
}

pub fn repositories_table(repositories: RepositoryList) -> Table {
    let mut table = Table::new();

    table.set_format(format());
    table.set_titles(row!["Name", "URI"]);

    for repository in repositories {
        table.add_row(row![
            repository.repository_name.unwrap_or("".to_owned()),
            repository.repository_uri.unwrap_or("".to_owned())
        ]);
    }

    table
}

fn format() -> TableFormat {
    FormatBuilder::new()
        .separator(LinePosition::Title, LineSeparator::new('-', '-', '-', '-'))
        .padding(1, 1)
        .build()
}
