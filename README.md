# GitHub Documentation Previewer

Web service that publishes a preview of a GitHub project documentation.

The documentation itself has to be built during the project continuous
integration and published as a GitHub actions artifact.

Then, the repository can be configured to request a preview when
a comment `/preview` is written in the pull request page. The request
is made as en HTTP POST request, for example:

```
curl -X POST https://docs-previewer-server.com/preview/pandas-dev/pandas/56907/
```

This will cause the server to download the artifact and extract it, so it is
available for example at https://docs-previewer-server.com/previews/56907/

The server will automatically delete old previews.

## Installation

A package is provided for Debian based systems. It can be installed with
the next command:

```
curl -sLO http://url/doc-previewer_0.1.0-1_amd64.deb \
    && sudo dpkg -i doc-previewer_0.1.0-1_amd64.deb \
    && rm doc-previewer_0.1.0-1_amd64.deb
```

To start the server:

```
nohub docs-previewer -c /path/to/config.toml > /path/to/log &
```

## Configuration

A sample configuration file is next:

```toml
previews_path = "/var/doc-previewer"
retention_days = 14
max_artifact_size = 524288000

[server]
address = "0.0.0.0"
port = 8000
publish_url = "https://doc-previewer.pydata.org/"

[github]
entrypoint = "https://api.github.com/repos/"
token = "xxxxx"
allowed_owners = [ "pydata", "pandas-dev" ]

[log]
level = "info"
format = "%a %{User-Agent}i"
```

All the fields are optional except for the GitHub token, which is required.
