# Dockerized Jepsen

This folder contains a containerized version of a Jepsen environment, consisting of one
"control node" container and 5 "node" containers which will run your system
under test.

The containers are orchestrated using [Docker
Compose](https://github.com/docker/compose). A script builds a
`docker-compose.yml` file out of fragments in `template/`, because this is the
future, and using `awk` to generate YAML to generate computers is *cloud
native*.

This Docker image attempts to simplify the setup required by Jepsen.
It is intended to be used by a CI tool or anyone with Docker who wants to try Jepsen themselves.

## Prerequisites

- **Docker**: Install Docker from the [official page](https://docs.docker.com/get-docker/)
- **Docker Compose**: Install Docker Compose from the [official page](https://docs.docker.com/compose/install/)

Note: You need docker-compose version >1.25 to allow gateway setting in the docker compose template (these versions merge the v2 and v3 file spec).
Also, don't use package manager version of docker (snap, apt etc.); use docker from the official downloads page.

## Quickstart

Run the following commands:

```
bin/up
bin/console
```

... which will drop you into a console on the Jepsen control node.

Your DB nodes are `n1`, `n2`, `n3`, `n4`, and `n5`. You can open as many shells
as you like using `bin/console`. If your test includes a web server (try `lein
run serve` on the control node, in your test directory), you can open it
locally by running using `bin/web`. This can be a handy way to browse test
results.

## Advanced

You can change the number of DB nodes by running (e.g.) `bin/up -n 9`.

If you need to log into a DB node (e.g. to debug a test), you can `ssh n1` (or n2, n3, ...) from inside the control node, or:

```
docker exec -it jepsen-n1 bash
```

During development, it's convenient to run with `--dev` option, which mounts `$JEPSEN_ROOT` dir as `/jepsen` on Jepsen control container.

Run `./bin/up --help` for more info.
