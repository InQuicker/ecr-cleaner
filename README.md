# ecr-cleaner

**ecr-cleaner** is a utility for deleting old Docker images in Amazon EC2 Container Registry (ECR).

## Legal

ecr-cleaner is released under the MIT license.
See `LICENSE` for details.

## Supported platforms

At this time, ecr-cleaner has only been developed for and tested on OS X. Support for Linux is planned.

## Installing dependencies

ecr-cleaner requires the following other programs to be available on your system:

* [OpenSSL](https://www.openssl.org/)

### OS X

All the dependencies can be installed with [Homebrew](http://brew.sh/):

```
brew install openssl
```

To use the Homebrew-installed OpenSSL, prefix the `cargo build` command (in the section on build from source below) with:

``` bash
OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
```
