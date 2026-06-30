# Security Policy

Native webviews run active web content inside the application process boundary
provided by the platform engine. Treat every URL and HTML string as untrusted
unless the embedding application controls it.

See [docs/security.md](docs/security.md) for the crate security model.

## Supported Versions

The crate is currently preparing an initial `0.1.0` release candidate. Security
support policy should be finalized before public publishing.

## Reporting

Do not include secrets, private keys, credentials, or sensitive user data in
issue reports or test fixtures. Use the project owner's preferred private
security contact once repository metadata is finalized.
