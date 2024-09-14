# lnrecc

A tool for streamlining recurring lightning payments. Weekly pocket money for your kids, perhaps?

## Prerequisites

You need to have gRPC API access to a LND lightning node you want to spend from, which includes a macaroon and a cert file.

## Installation

Download the archive from the release page, optionally verify the signature, extract it, update the `config.yaml` with your configuration and run the executable. Or you can clone the repository and build it yourself. Or you can run in docker.

## Upcoming features

- Ability to spend from Core Lightning
- Bolt12
- Posibility to set up notifications after every payment
- Specify timezones
- Whatever else comes up

## Contributing

Open to PRs!
