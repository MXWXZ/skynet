# frontend-dev
## Bug fix
1. Fixed a bug where the copyright could be hidden when the content height exceeded the page limit.

# v0.2.3
## Changes
1. `frontend`: Support i18n for status.
2. `frontend`: Dependency upgrade.
3. `skynet`: `skynet_api` logs are now renamed to `skynet`.
4. `skynet`: Dependency upgrade.
5. `skynet`: 403 body will be dropped.

## Bug fix
1. `skynet`: Fix returning null fields.

# v0.2.2
## Bug fix
1. Fix postgres backend bug.

# v0.2.1
## Bug fix
1. Fix `no process-level CryptoProvider` for rustls 0.23. 

# v0.2.0
## Breaking changes
1. `skynet` crate is split to `skynet` and `skynet_api`, plugins now should depend on `skynet_api`.
2. Based on new framework `actix-cloud`.
3. Most system is re-designed.

## Changes
1. Only warning and error logs will increase the unread count.
2. Allow change root username.
3. `/ping` is renamed to `/health`.

## Bug fix
1. Fix a bug that prevent success logs written to the database.

# v0.1.1
## Changes
1. `listen.ssl` will only raise warning when `proxy.enable` is `false`.

## Bug fix
1. Fix a IP parsing bug when proxy is enabled.
2. Fix CSP violation for reCAPTCHA.
3. Fix validation failed for reCAPTCHA.
4. Fix login db error when using PostgreSQL.
5. Fix plugin segmentation fault in some systems.
6. Fix segmentation fault when shared API is enabled.

# v0.1.0
First version of skynet!