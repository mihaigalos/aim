# Changelog

All notable changes to this project will be documented in this file.

## [1.8.6] - 2025-03-12

### 🐛 Bug Fixes

- Clippy - Warnings
- S3 - Docker
- Clippy - Warnings
- Clippy - Warnings
- Cargo - Update
- Clippy - Warnings
- Driver - Add small delay upon starting container with keys
- Dockerfile
- Justfile - Seq syntax
- Docker-compose - Deprcate docker-compose in favor of docker compose
- S3 - API breakage
- Clippy - Warnings

### ⚙️ Miscellaneous Tasks

- Tests - Increase verbosity
- Tests - Increase verbosity
- Logs - Increase verbosity
- CI - Trigger with latest upstream changes
- CI - Trigger with latest upstream changes
- CI - Trigger with latest upstream changes
- Bump version to 1.8.6

## [1.8.5] - 2023-09-09

### 🐛 Bug Fixes

- Hash - Naming of hashed file

### ⚙️ Miscellaneous Tasks

- Bump version to 1.8.5

## [1.8.4] - 2023-09-07

### 🐛 Bug Fixes

- Naming - Rename LICENSE.md to LICENCE.md
- Docker

### 💼 Other

- Sync latest lock
- Hosting port
- Update to newest by running cargo update
- Add image size
- Install
- Setup in Justfile
- Fix tests
- Use own forks to prevent leaks
- Use 2 space YAML files
- Update maintainer email
- Yaml formatting
- Spelling
- Autofix if single suggestion
- Check filenames

### ⚙️ Miscellaneous Tasks

- Pre-commit - Split into blocks
- Bump version to 1.8.4

## [1.8.3] - 2023-04-30

### 💼 Other

- Use v1.0.4
- Fix updated LICENSE.md
- Clippy warnings
- Fix warnings
- Reference centralized mihaigalos/workflows
- Fully automate
- Healthchecks
- Use host network when building
- S3
- Unify with docs implementation
- Simplify get
- S3
- Use new GitHub syntax for outputs
- Update
- Bump to 1.8.3
- Fix warnings in latest

## [1.8.2] - 2023-01-21

### 💼 Other

- Update to newest by running cargo update

## [1.8.1] - 2023-01-07

### 💼 Other

- Cluster similar
- Improve wording
- Simplify demo screencast
- Sync latest lock
- Clippy warnings
- Reference

## [1.8.0] - 2022-10-27

### 💼 Other

- Use latest clap
- Use latest clap
- Run on yaml change
- Use get_flag() instead of contains_id()
- Add --version
- Use autoclap 0.3.0
- Fix arg action, should default to set true
- Reconcile conflicts
- Use url-parse v1.0 API
- Resolve conflicts
- Migrate to dotenvy
- Superfluous dep to openssl-src
- Fix missing COPYRIGHT_YEARS

## [1.7.2] - 2022-09-25

### 💼 Other

- Bump rustup update to 1.64

## [1.7.1] - 2022-09-24

### 💼 Other

- Improve wording
- Add interactive gif
- Test missing paths
- Test missing paths
- Test missing paths
- Test missing paths
- Test missing paths
- Clippy warnings
- Ssh and ssh_auth
- Sftp
- S3
- S3
- Netrc
- Https
- Io
- Https
- Hash
- Ftp
- Driver
- Driver
- Bar
- Address
- S3
- Driver
- Bar
- Address
- Add clippy check
- Fail on clippy warnings
- Update badge
- Reference centralized repository
- Propagate secrets to centralized pipeline
- Reference centralized repository
- Add badge
- Update badge
- Bump rustup update to 1.64

## [1.7.0] - 2022-08-10

### 💼 Other

- Fix tests
- Split and not cover navigate() since interactive
- Update tolerated vulnerabilities
- Improve feature listing
- Describe interactive mode

## [1.6.0] - 2022-08-08

### 💼 Other

- Hash handlers
- Successul non-final implementation of hashed get
- Non-final implementation of hashed put
- Add FTP to hashed handlers
- Add SFTP to hashed handlers
- Add SSH to hashed handlers
- Add S3 to hashed handlers
- Switch get to using hash handlers
- Switch put to using hash handlers
- Split extract_scheme_or_panic()
- Parse scheme from input

## [1.5.3] - 2022-07-24

### 💼 Other

- Fix just tests
- Add dep to crate
- Handle slash
- Add Enter and Esc keys
- Implement list()
- Split navigate() and finish_navigation()
- Implement get_links()
- Split to dispatch()
- Internalize nagigate() and finish_navigation()
- Loop until navigation finished
- Navigate subpath
- Tab completion
- Navigate out of
- Use
- Enable commandline feature
- Consume input instead of hard-coded address URL

## [1.5.2] - 2022-07-16

### 💼 Other

- Print info when not using anonymous credentials
- Move towards API convergence
- Use latest 0.5.1
- Use latest 0.5.2
- Bump version to 0.5.4
- Test parse works when not silent

## [1.5.1] - 2022-07-16

### 💼 Other

- Initial commit
- Split into table
- Add generating docker command
- Fix Dependabot alert number 11
- Centralize in common.sh
- Centralize in common.sh

## [1.5.0] - 2022-06-23

### 💼 Other

- Prepare for returning listing
- No execution on wip label
- Only run on PRs
- Only run on PRs
- Job as step
- Use abstract Self instead of duplicating name
- Use fork for parsing url
- Improve verbosity
- Working GET
- Make bar visible
- Use trait to enable resume
- Initial untested PUT
- Seek in local file
- Cleanup imports
- Split parse_args()
- Get already uploaded bytes
- Seek in input for the amount of bytes already transfered
- Test uploading to subfolders
- Adapt getting curl response codes for subfolders
- Test GET and PUT
- Create file if not existing during PUT

## [1.4.0] - 2022-06-12

### 💼 Other

- Initial commit
- Explain output in README.md
- Improve wording
- Improve AWS credentials parsing verbosity
- Test conversion to std::io::Error
- Simplify http serving via Warpy
- Test

## [1.3.1] - 2022-06-08

### 💼 Other

- Simplify building dockers
- Enable debugging
- Fix identing
- Fix minio IP
- Fix pipeline
- Split run steps
- Use Alpine 3.16
- Use native machine instead of buildx with QEMU
- Pull before building
- Reuse buildx
- Pin buildx setup jobs
- Add swapspace to prevent exit code: 137 when building dockers
- Bump to 1.3.1

## [1.3.0] - 2022-06-07

### 💼 Other

- Improve
- Initial commit
- Proper _start and _stop
- Successful initial connection
- Remove runtime errors
- List all files, folders
- Split new_storage()
- Use separate structure
- Split new_storage()
- Split put_string()
- Split get_string()
- Split _print_bucket_location()
- Split {_get_tags,_set_tags}()
- Improve naming
- Split {_get,_put}_binary
- Ensure file overwrite during put_string
- Inject args via formal parameters
- Prepare list integration via get()
- List
- Successful list
- Determine if host has capability
- Split _get_transport()
- Auto allow http in debug mode
- Split _get_header()
- Remove superfluous
- Consolidate
- Fix tests
- Mock TLS
- Test Storage::new()
- Remove superfluous
- Fix hard-codings
- Mock where necessary
- Test happy paths
- Remove facade for simplification
- Split get_bucket()
- Debug
- Non-resumable transfers
- Split list()
- Working get()
- Fix tests
- Working list()
- Use self-hosted instead of restricted public
- Move test_get_ftp_resume_works() to driver
- Remove superfluous
- Improve naming
- Fix nasty sequencing
- Remove ftp.fau.de references
- Debug
- Checkout to ensure hashes ok
- Cover _get_string and _put_string
- Cover _get_string and _put_string
- Cover S3 get
- Put before Get of string
- Fix tests
- Split setup()
- Add put()
- Working put()
- Add i-test
- Add i-test
- Fix #1 Infinite loop in BN_mod_sqrt reachable when parsing certificates
- Mixin AWS credentials
- Use get_credentials()
- Improve wording
- Test env vars
- Test_get_credentials_works_when_tyipical
- Test_mixin_aws_credentials_from_aws_folder_works_when_typical
- Ignore result error if folder non existent
- Bump to 1.3.0
- Upgrade to rustc 1.59

### 🧪 Testing

- Add contents

## [1.1.3] - 2022-05-22

### 💼 Other

- Upgrade to 0.2.1
- Stop main test loop on errors
- No surpression of output if check requested
- Fix
- Remove duplicate

## [1.1.2] - 2022-04-15

### 💼 Other

- Improve
- Simplify
- Installation
- Templetize maintainer and description
- Initial commit
- Fix tests
- Improve wording when serving folder
- Bump to 1.1.2

## [1.1.1] - 2022-04-02

### 💼 Other

- Improve structure
- Add examples for hosting and downloading

## [1.1.0] - 2022-03-27

### 💼 Other

- Improve README.md
- Fix resume
- Improve
- Restructure sections
- Pin port to 8082
- Pin port to 8082

## [1.0.0] - 2022-03-23

### 💼 Other

- Use asymmetric keys as well as user/pass
- Explicit info message to use dep instead
- Ensure parse works when ssh user and no pass
- Successful just bash tests

<!-- generated by git-cliff -->
