# yage: yaml age

A simple tool to manage encrypted secrets in YAML files with age encryption.

`yage` is using [age encryption](https://github.com/FiloSottile/age) to encrypt the *values*
in a [YAML](https://yaml.org/) file while keeping the *keys* unchanged.

A simple yaml file like this one:

```yaml
backend:
  url: https://example.com
  username: gaspard
  password: api_s3cr3t_k3y
```

is encrypted by `yage` to:

```yaml
backend:
  url: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBtNGppeGdDRUJiVm9JcE9iTG1KYWxzWmx1Um1IVEVXTmx5WkhEVmk1TmpBCnh5b05IYzZOdUc3cG5qTElIei9LVmZJQk1sZkZFQUJncXRxc2huWXVRSzAKLT4gKmUtZ3JlYXNlIGBOTjkgIUY8UClwTXUKZU9oaVp5cjUrR2NENDN5UEJJbDJTeVU0UllJcjAyRTFySXVXdDQzU2poaVJJZ3EvQy9UMFk5UHdxNFAyK2VXRwpTTDYwOFlQM1ptem5qMzgxclREUVpQWERXbXp4elB0ck9UczJvNTlEVk1SdjM5b1MvZmNrN3p1Q0I0QTlLMThJCmIrOTMKLS0tIHNFNXpPT2VnblZKVCtTZTR2dmFDT3VzeE1WSFdzamtBTzQ0aGM1U2RCSjAKfS0+H2nU0KZ5EscjEXOhcth+2tTaXm33uKZcWMB9+P2p1fn2+SdO9FlH7rGDn2lo8mq7Yg==]
  username: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBDUjV1STI5Y2JLK3V4ZjlEeTg3aWNpVFl2UVJKcVpoL0dwTTVzdnNRZ0dNClpGM214MkIwQlhWdG0yeG8zcWFNM2lxeW5qWEY2cENVVTlLK2lyQWh1SFUKLT4gQVBDU2k1LWdyZWFzZSBnVjBgICcmUjIgOXs/a29+cyBAInBKTjkvPgo2ODN5bEttVGxlNE1nSTlPRVFGQzNnT0pneWU4cnkwdzRXaC9UT21LZS81TzNiSTZSMVRZVVBuWUJMcjNnMjhUCmQvenZvTWRRRUhjTDBvUksvZlN0RTlxRmpGVE5EcVV2ME1TNTVpT2lCKzNEUmtPRmNySkR2dHRTdncKLS0tIGxqbDNtTktPNmxWV25zd2V2MGpMRE8yOUx4Rm0zdmltZk9JRTdmQ29MTFEKZHImd+aw9OcLyHAPvBUyhDy/9bkupIKFOhkO/MYD6IoOtKly1fQqTA==]
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSB2eXFiMTFMV1ZpTTh6QVQzRUo0anI3TVUxVEJTNE01UW1xcEFvdHJZcVFJClNyNllQT2hmM0lwWldXZllHWGtUQ3JZeW1sOHJ0b2Vabk5YZ3o5a2dOeG8KLT4gfC1ncmVhc2UgMSBEbyFqMyAiT2YnIk80Cnp1WVFoTkxSaGc2dXNOaGpRWGVRNkU0WlFHeUhzd0UKLS0tIERHNmN3WldFcHhpTmczdzhGOVF0bld1ZHQxbi8yY3o5b2F4ZExUU0l6OFkKq01qW5HZMk5O/tHAMP4ezAeb38DX0+8vle29hNpZvoVLSw/wsHu0yNlcp053kFQ=]
```

Having the keys unencrypted allows to easily manage the file in a version control system, like git,
and to use this file in a CI/CD pipeline or in a [gitops](https://en.wikipedia.org/wiki/DevOps#GitOps)
workflows.

If you think this looks a lot like [SOPS](https://getsops.io/), you're right! This is basically what
SOPS is doing, but we some key differences:
* `yage` doesn't include any metadata in the encrypted file, in particular no [MAC](https://en.wikipedia.org/wiki/Message_authentication_code).
* `yage` is focused on age encryption, and include everything required in a single binary.

The lack of MAC, while it could be seen as a missed opportunity to add some security, actually allows
some interesting use cases:
  * the encrypted file can modified by someone that only has the public key, while still preserving
    the encrypted values.
  * the encrypted file can modified by multiple persons and merged in a version control system without
    having to decrypt it first.
  * the encrypted file only contains the original keys and the encrypted values, so it can be used
    to verify that it is usable for a specific task without having to decrypt it or remove the metadata.

## Installation

Only available from source for now. More to come soon!

### From source

Just run

```sh
$ cargo install --path .
```

in this repository

## Command line reference

`yage` comes with a full description of its commands and options. Just run `yage --help` to get it.

```sh
$ yage --help
A simple tool to manage encrypted secrets in YAML files.

Usage: yage [OPTIONS] [COMMAND]

Commands:
  decrypt  Decrypt the values in a YAML file
  edit     Edit an encrypted YAML file
  encrypt  Encrypt the values in a YAML file
  env      Execute a command with the environment from the encrypted YAML file
  keygen   Generate a new age key
  pubkey   Convert private age keys to their public key
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...          Increase logging verbosity
  -q, --quiet...            Decrease logging verbosity
      --completion <SHELL>  Generate the completion code for this shell [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help                Print help
  -V, --version             Print version
```

See also the [markdown version of the command line reference](doc/CommandLineHelp.md).

You may also find convenient to install the completion for your shell. For example for fish:

```sh
$ yage --completion fish > ~/.config/fish/completions/yage.fish
```

## Usage

First generate a new age key pair:

```sh
$ yage keygen -o prod.key -p prod.pub
Public key: age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6
```

The public key can be shared with anyone. It allows everybody that has that key to encrypt a secret that
can be decrypted only by someone who has access to the private key. The private key must be kept secret.

Both keys are just text:

```sh
$ cat prod.key
AGE-SECRET-KEY-1EZEU9RUTW3K5GV98ER6RHMS73QJNQ37ARWG6MWHXM4JP8FVD3A9QK2DD70
```

The public key could be committed to a git repository:

```sh
$ git add prod.pub
$ git commit -m "Add prod public key"
```

Make sure that the private key won't be committed by mistake in the repository, for example by adding it
to the `.gitignore` file, and by using a tool like [gitleaks](https://github.com/gitleaks/gitleaks).

```
$ echo "*.key" > .gitignore
```

The private key should be kept in a secure place, for example in a password manager. It may also
be added to a CI/CD pipeline as [a secret](https://docs.github.com/actions/security-guides/encrypted-secrets).

Once you have a private and a public key, you can encrypt a YAML file. The `--recipient-file` or `-R`
option is used to specify a file containing the public keys to use for encryption. The recipients
can also be specified directly on the command line with the `--recipient` or `-r` option.

```sh
$ yage encrypt --recipient-file prod.pub secrets.yaml --output secrets.enc.yaml
```

If you prefer you can encrypt the file in place wit the `--in-place` or `-i` option:

```sh
$ yage encrypt -iR prod.pub secrets.yaml
```

You need the private key to have access to the decrypted values, so if you don't have it,
the encrypted file is showing you what is encrypted, for example `backend.password`, but not the values.

```yaml
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSB2eXFiMTFMV1ZpTTh6QVQzRUo0anI3TVUxVEJTNE01UW1xcEFvdHJZcVFJClNyNllQT2hmM0lwWldXZllHWGtUQ3JZeW1sOHJ0b2Vabk5YZ3o5a2dOeG8KLT4gfC1ncmVhc2UgMSBEbyFqMyAiT2YnIk80Cnp1WVFoTkxSaGc2dXNOaGpRWGVRNkU0WlFHeUhzd0UKLS0tIERHNmN3WldFcHhpTmczdzhGOVF0bld1ZHQxbi8yY3o5b2F4ZExUU0l6OFkKq01qW5HZMk5O/tHAMP4ezAeb38DX0+8vle29hNpZvoVLSw/wsHu0yNlcp053kFQ=]
```

But with just the public key, you can still modify the file, for example to add a new secret:

```yaml
mail:
  apiKey: my_secret_key_to_send_emails
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSB2eXFiMTFMV1ZpTTh6QVQzRUo0anI3TVUxVEJTNE01UW1xcEFvdHJZcVFJClNyNllQT2hmM0lwWldXZllHWGtUQ3JZeW1sOHJ0b2Vabk5YZ3o5a2dOeG8KLT4gfC1ncmVhc2UgMSBEbyFqMyAiT2YnIk80Cnp1WVFoTkxSaGc2dXNOaGpRWGVRNkU0WlFHeUhzd0UKLS0tIERHNmN3WldFcHhpTmczdzhGOVF0bld1ZHQxbi8yY3o5b2F4ZExUU0l6OFkKq01qW5HZMk5O/tHAMP4ezAeb38DX0+8vle29hNpZvoVLSw/wsHu0yNlcp053kFQ=]
```

You can encrypt the new secrets by just using the same command as before:

```sh
$ yage encrypt -iR prod.pub secrets.yaml
```

`secrets.yaml` now contains the encrypted values:

```yaml
mail:
  apiKey: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBWbnRCSVhYWmNKaUg1QUgwSjFOK0JITWdkdktXT2wwbmtIOExrTE5rWG53ClpwOE5WYzdTZnBZandnM2RyVzFMNDRkQTFBL281WE5URW55bytUYk5mdFkKLT4gezdgcUA2WiMtZ3JlYXNlIF9CTDFAcE4/CkRFVzU0Vmt6RDBtdnhoZldhQmJld2JuMDUzVnNGRkczYTJEVnF0TWVWTmdmcit4TDFzU1pTYTh6NW51cDlRNmwKQ3cKLS0tIGx1Rm1BNlpFejNDSm0rejVSRGJIQjlUS3hOTXFBd2dRcXE1eFhTUjAwRGsKe59C1Is1S3EEEvxyqaVz4ZLWbheaa/i7xDv6fJAC2AkFmLGRd0VuhcPK3AUpy2V64MQrVmmGTZabt2Jc1w==]
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSB2eXFiMTFMV1ZpTTh6QVQzRUo0anI3TVUxVEJTNE01UW1xcEFvdHJZcVFJClNyNllQT2hmM0lwWldXZllHWGtUQ3JZeW1sOHJ0b2Vabk5YZ3o5a2dOeG8KLT4gfC1ncmVhc2UgMSBEbyFqMyAiT2YnIk80Cnp1WVFoTkxSaGc2dXNOaGpRWGVRNkU0WlFHeUhzd0UKLS0tIERHNmN3WldFcHhpTmczdzhGOVF0bld1ZHQxbi8yY3o5b2F4ZExUU0l6OFkKq01qW5HZMk5O/tHAMP4ezAeb38DX0+8vle29hNpZvoVLSw/wsHu0yNlcp053kFQ=]
```

Note that `backend.password` has not been re-encrypted, so you can easily track the changes in your
version control system.

To decrypt the file, you need the private key:

```sh
$ yage decrypt --key-file prod.key secrets.enc.yaml --output secrets.yaml
```

Or just do it in place:

```sh
$ yage decrypt -iK prod.key secrets.yaml
```

If your decrypting in a CI/CD pipeline, you may find convenient to put the private key in the
`YAGE_KEY` environment variable. This way you can just use `yage decrypt -i secrets.yaml`.

If you have the private key, you can edit the file in place with your favorite text editor
configured in the `EDITOR` environment variable:

```sh
$ export EDITOR=vim
$ yage edit -K prod.key secrets.yaml
```

The file is edited in clear in the editor and re-encrypted when you save and quit. Here again only
the modified values are re-encrypted. The others are left unchanged to allow easy tracking of changes.

Finally, with the private key, you can use the secrets in the encrypted file to run a command
with the environment variables set to the decrypted values in a single command:

```sh
$ yage run -K prod.key secrets.yaml env terraform apply
```

## Why?

Mostly to unlock the ability to add values to an encrypted file without having to decrypt it,
thing that is not possible with SOPS. Something I've not been the only one frustrated with, see
[here](https://github.com/getsops/sops/discussions/1081),
[here](https://stackoverflow.com/questions/74103453/is-it-possible-to-update-a-sops-encrypted-file-without-decrypting-it-first),
[here](https://github.com/getsops/sops/issues/1117), [here](https://github.com/getsops/sops/issues/833), …

And because writing command line tools in rust is fun!

## Still to be done

* [ ] Add tests. Coming soon!
* [ ] Add a status command to ensure the whole file is encrypted/decrypted
* [ ] Support comments. Sadly no YAML library that I know of supports comments, so this will be a bit tricky.

## License

`yage` is distributed under the terms of the MIT license.

See [LICENSE](LICENSE) for details.
