# ecr-cleaner

**ecr-cleaner** is a command line utility to delete old images in an Amazon EC2 Container Registry (Amazon ECR).

## Usage

With no subcommands:

```text
USAGE:
    ecr-cleaner [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --profile <profile>    AWS credentials profile name
    -r, --region <region>      AWS region to operate in [default: us-east-1]

SUBCOMMANDS:
    clean    Delete images from a repository
    help     Prints this message or the help of the given subcommand(s)
    list     List ECR repositories or their contents
```

The `clean` subcommand:

```text
USAGE:
    ecr-cleaner clean <repository> --count <count> --threshold <threshold>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --count <count>            The number of images to delete if the threshold is met
    -t, --threshold <threshold>    The number of images that must exist before any will be deleted

ARGS:
    <repository>    ECR repository whose images will be deleted
```

The `list` subcommand:

```text
USAGE:
    ecr-cleaner list [repository]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <repository>    ECR repository to list the contents of
```

## Legal

ecr-cleaner is released under the MIT license.
See `LICENSE` for details.
