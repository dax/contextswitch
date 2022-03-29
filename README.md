# Contextswitch

[![Apache 2 License](https://img.shields.io/badge/license-Apache 2-blue.svg)](https://www.apache.org/licenses/)  
[![codecov](https://codecov.io/gh/dax/contextswitch/branch/main/graph/badge.svg?token=CHUSURDB5Q)](https://codecov.io/gh/dax/contextswitch)
![CI](https://github.com/dax/contextswitch/actions/workflows/ci.yml/badge.svg)

Contextswitch is a todo list application linking bookmarks to a task.
Integrations with third parties applications add context to a task:
- a link to a note taking application to add notes to a task
- a link to a Slack thread
- a link to a Github issue, pull request or discussion related to the task
- ...

It is intended to be based on existing todo applications and augment them.

## Features

- [X] list tasks
- [ ] add a task
- [ ] add a bookmark to a task
- [ ] augment a task with third party integration
- [ ] update a task status (waiting, done, ...)
- [ ] schedule a task
- [ ] update a task status based on bookmarks notifications

### Integrations

Todo application backend:
- [X] taskwarrior
- [ ] todoist

Third parties integrations:
- [ ] Github
- [ ] Slack

Frontend integrations:
- [X] Contextswitch
- [ ] [Sidebery](https://github.com/mbnuqw/sidebery) Firefox add-ons
 
## Installation

### Using cargo (for development)

```bash
cargo make run
```

### Manual

1. Get the code

```bash
git clone https://github.com/dax/contextswitch
```

2. Build api and web release assets

```bash
cargo make build-release
```

It will produce a `target/release/contextswitch-api` backend binary and frontend assets in the `web/dist` directory.

3. Deploy assets

```bash
mkdir -p $DEPLOY_DIR/config
cp -a target/release/contextswitch-api $DEPLOY_DIR
cp -a web/dist/* $DEPLOY_DIR
cp -a api/config/{default.toml, prod.toml} $DEPLOY_DIR/config
```

4. Run server

```bash
cd $DEPLOY_DIR
env CONFIG_FILE=$DEPLOY_DIR/config/prod.toml ./contextswitch-api
```

### Using Docker

#### Build Docker image

```bash
docker build -t contextswitch .
```

#### Run Contextswitch using Docker

```bash
docker run --rm -ti -p 8000:8000 contextswitch
```

## Usage

Access Contextswitch using [http://localhost:8000](http://localhost:8000)

## License

[AGPL](LICENSE)
