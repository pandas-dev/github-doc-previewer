# GitHub Documentation Previewer

Web service that publishes a preview of a GitHub project documentation.

The documentation itself has to be built during the project continuous
integration and published as a GitHub actions artifact.

## Client installation

To be able to use this service in your project, you need to have a CI
job that looks like this:

```yaml
on: [ push, pull_request ]

jobs:
  doc:
    name: Build Documentation
    runs-on: ubuntu-latest
    steps:
      # Instead of this step you may have a step to check out the repository,
      # another to call `sphinx-build`, or anything that generates a directory
      # with the html version of your website or documentation
      - run: mkdir docs_dir && echo "Preview is working!" > docs_dir/index.html

      - uses: actions/upload-artifact@v4
        with:
          name: docs_artifact
          path: docs_dir
```

Then, you can add one action in your CI to for example publish the website or
documentation of a pull request when a user writes a comment with the content
`/preview`:

```yaml
on:
  issue_comment:
    types: created

jobs:
  previewer:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
    if: github.event.issue.pull_request && github.event.comment.body == '/preview'
    steps:
      - uses: pandas-dev/github-doc-previewer@master
        with:
          # The server specified here has to explicitly allow your GitHub organization
          previewer-server: "https://pandas.pydata.org"

          # Note that this has to match with the name of the job where the
          # `upload-artifact` action is called
          artifact-job: "Build Documentation"
```

## Server Installation

Currently only packages for Debian are provided. The package can be installed
using the next command:

```
curl -sLO https://github.com/pandas-dev/github-doc-previewer/releases/download/v0.3.0/doc-previewer_0.3.0-1_amd64.deb \
    && sudo dpkg -i doc-previewer_0.3.0-1_amd64.deb \
    && rm doc-previewer_0.3.0-1_amd64.deb
```

To start the service:

```
sudo systemctl enable doc-previewer.service
sudo systemctl start doc-previewer.service
```

### Configuration

A sample configuration file is next:

```toml
previews_path = "/var/doc-previewer"
retention_days = 14
max_artifact_size = 524288000

[server]
address = "0.0.0.0"
port = 8000
url = "https://doc-previewer.pydata.org/"

[github]
entrypoint = "https://api.github.com/repos/"
token = "xxxxx"
allowed_owners = [ "pandas-dev", "pydata" ]

[log]
level = "info"
format = "%a %{User-Agent}i"
```

All the fields are optional except for the GitHub token, which is required.

### Logging

When a problem exists, the service will in general report it to the client.
Checking the logs of the `github-doc-previewer` action run that failed may
provide information on the cause of the problem. Errors will also be logged
in the server, and can be checked using the standard journal logs:

```bash
journalctl -u doc-previewer.service
```
