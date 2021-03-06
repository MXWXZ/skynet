# skynet

[![Go Report Card](https://goreportcard.com/badge/github.com/mxwxz/skynet)](https://goreportcard.com/report/github.com/mxwxz/skynet)

Skynet is a service integration and management system, especially optimized for personal and home-lab use. With plugin support, you can easily embed whatever software you want to satisfy your need.

Security is considered as **Tier 0** priority in Skynet, we will not consider features conflict with our security policy. If you find vulnerabilities, please report ASAP.

## Quick start

Skynet master program only support Linux system, we recommand docker image for simple start up.

1. `mkdir skynet && cd skynet`
2. Copy `conf.yml` and `docker-compose.yml` to the folder
3. `vim conf.yml` to modify your config, you **MUST** change redis address config to `redis:6379`
1. `touch data.db` or copy your exist database
2. `docker-compose up -d` to start skynet
3. visit `localhost:8080`

### Create initial user

You need to use the command line to add your initial user, you may add more user in web UI or use command line for batch add.
Note that `--root` is needed to give you root privilege, by default no permission is allowed.

    skynet user add $username --root

For security reasons, we do not support specify certain initial password, you must save the password shown on screen, or use

    skynet user reset $username

to reset the password.

## Plugins

You can find plugins in our [official support plugin](plugin) or other user shares.

**!!Please note that all plugins have the same privilege as skynet, use trusted plugins ONLY!!**

## Develop

See [develop note](develop.md)

## Reference Project

see [go.mod](go.mod)
