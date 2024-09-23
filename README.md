# lnrecc

A tool for streamlining recurring lightning payments. Weekly pocket money for your kids? Perhaps a subscription or a regular donation? Up to you!

## Prerequisites

You need to have a gRPC API access to LND lightning node you want to spend from, which includes a macaroon and a cert file.

## Installation

Download the archive from the release page, optionally verify the signature, extract it, create the `config.yaml` with your configuration and run the executable. Or you can clone the repository and build it yourself. Or you can run it in docker.

Optional CLI parameters:

- `--config-path <path>`
- `--log-path <path>`

### Software verification (optional, but recommended)

1. Import gpg key. `gpg --keyserver hkps://keys.openpgp.org --recv-keys 54650F2C495A04B2927EDF729936610A65758899`
2. Verify signature. `gpg --verify lnrecc-*.sha256.txt.sig` MUST say `Good signature`
3. Verify hashes. `shasum -a 256 --ignore-missing --check lnrecc-*.sha256.txt` MUST say `lnrecc-*.tar.gz OK`

### Default config

```
macaroon_path: "/home/my-user/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
cert_path: "/home/my-user/.lnd/tls.cert"
server_url: "https://localhost:10009"
jobs:
  - name: "My first job"
    schedule: "0 30 9,12,15 1,15 May-Aug Mon,Wed,Fri 2018/2"
    amount_sats: 10000
    ln_address_or_lnurl: "nick@domain.com"
    max_fee_sats: 5
    memo: "Scheduled payment coming your way!"
```

Currently, `server_url` mustn't be an IP address, otherwise you run into `InvalidDNSNameError`. If lnd is not running locally and you don't have a domain name for it, add `<ip address> lnd` to your `/etc/hosts` and then in config.yaml use `server_url: "https://lnd:10009"`

`schedule` is a cron-like syntax extended by seconds field (first one) and optionally also year. Here are a few examples:

- `0 * * * * *` run every minute
- `0 */5 * * * *` run every 5 minutes
- `0 30 12 * * *` run every day at 12:30 PM
- `0 0 9 * * 1` run every Monday at 9:00 AM
- `0 0 6 1 * *` run on the first of every month at 6:00 AM

All job times are currently in UTC.

## Upcoming features

- Ability to spend from Core Lightning
- Bolt12
- Posibility to set up notifications after every payment
- Specify timezones
- Whatever else comes up

## Contributing

Open to PRs!
