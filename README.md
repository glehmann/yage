# yage: yaml age

A simple tool to manage encrypted secrets in YAML files with age encryption.

`yage` is using [age encryption](https://github.com/FiloSottile/age) to encrypt the _values_ in a
[YAML](https://yaml.org/) file while keeping the _keys_ unchanged.

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
  url: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSA1KzZiNVVvTThaQ0dSbVZBMElRUEpZRlVxb1VaamU3SGtWRWxseFBHMkVNClBsUGJMWFJ0MnA0czA0MEE0VnRjY3Q3VnE4NElpUHQ2aTNKUlVMOUJnbEUKLT4gZGVVYFRQcVwtZ3JlYXNlIHR+OSVVZyMhCituclJ0ZTZJU1grSmpxaTZvb2hMYlBvMnpIbGFja0ltRm9BdE9sTVJ5RG14RkhNaXNsc0xUbUViT1lyRVh6dWYKdHRsVU10SHJ1YytaY2hhdWdjQ0lJVnFkczJFcHBMMXl3QTNEQUc1SW9YM0hscGZVbEdPdWZWTQotLS0gcTVIZzZrdnVJNGNoNm51UEE1alJHVmJsQnRMdStuVXU0Q1pJSzFzVlU1QQqH64hS0UV4JI6CtYHpSCEslxLqi3764zaxF/VJy67gQOBrj2TV1z6NwJaBmlUPBQB2zROq|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
  username: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBVU2tuOWFpV2NDT05DSC9FbHNadzVVTTJmSFQwekRjRjZiOERDVTZ0aGdnCnhKTGpQcGY1SUw0bFd2cFhudC9yY1I0VTJxWklBYUkvYlhzNHhUeCtIVmsKLT4gTGJ+LWdyZWFzZSAhClJlTU5TZ3NldGRaRzRGQThOZmMrdkFnS0NzUnBpMmFpRTM4d0hGQXEKLS0tIGQvSXIxcXpDYk9yTU9ERDlEMHV1d0VmTGovWU9SVkJ5TkswYS9rMmdGbHMK3GehcWJL1GUKFGLJXQFGUQLhs56mgHRYpBIsFgYpyW818dG27bz1jQ==|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSAyRVUyTHpmQnZuTlc5MGJhODFuSE5WMmdzalNid0ExamliNDREbWlJUXdvClFhS3JGSGV2cjlnL0dWUnNwSDNvWGVHUDVDendkNzFWWXJCcTNDTVJNLzgKLT4geW0tZ3JlYXNlIF8zJ3MpXksKeXRwMW9aeitydwotLS0gWVNjbnZsUmNRWTdtM0pjVjNKQjBDZ1k2cVNhcWkwMnAxREwwMXptREhLZwrpCkrMFiq/XWfAyFRrLuLkkEPhnZ9Kt68pg5ENgDTV9+3iRcy6XKYdkqnEBRidMg==|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
```

Having the keys unencrypted allows to easy management of the file in a version control system, like
git, and to use this file in a CI/CD pipeline or in a
[gitops](https://en.wikipedia.org/wiki/DevOps#GitOps) workflows.

If you think this looks a lot like [SOPS](https://getsops.io/), you're right! This is basically
what SOPS is doing, but with some key differences:

- `yage` doesn't include any metadata in the encrypted file, particularly no
  [MAC](https://en.wikipedia.org/wiki/Message_authentication_code).
- `yage` is focused on age encryption, and includes everything required in a single binary.

The lack of MAC, while it could be seen as a missed opportunity to add some security, actually
allows some interesting use cases:

- The encrypted file can be modified by someone who only has the public key, while still preserving
  the encrypted values.
- The encrypted file can be modified by multiple persons and merged in a version control system
  without having to decrypt it first.
- The encrypted file only contains the original keys and the encrypted values, so it can be used to
  verify that it is usable for a specific task without having to decrypt it or remove the metadata.

## Installation

### From binaries

Go to the [releases page](https://github.com/glehmann/yage/releases), download the binary for your
platform, extract it, and put the `yage` binary in a directory in your `PATH`.

For example, on linux with an intel/amd64 processor, you can run the following commands to install
`yage` in `~/.local/bin`:

```sh
curl -ssL https://github.com/glehmann/yage/releases/download/0.5.0/yage-0.5.0-linux-amd64.tar.gz | tar xzf - -C ~/.local/bin --strip-components=1
```

### Docker

`yage` is also available as a [docker image](https://github.com/glehmann/yage/pkgs/container/yage).

Here is how you can use it to encrypt a file in place:

```sh
docker run --rm -t -v $(pwd):/src ghcr.io/glehmann/yage:0.5.0 encrypt -iR prod.pub secrets.yaml
```

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
A simple tool to manage encrypted secrets in YAML files with age encryption

Usage: yage [OPTIONS] [COMMAND]

Commands:
  check       Check the encryption status of a YAML file
  decrypt     Decrypt the values in a YAML file
  edit        Edit an encrypted YAML file
  encrypt     Encrypt the values in a YAML file
  env         Execute a command with the environment from the encrypted YAML file
  keygen      Generate a new age key
  pubkey      Convert private age keys to their public key
  recipients  List the recipients of the encrypted data
  re-encrypt  Re-encrypt the values in a YAML file
  help        Print this message or the help of the given subcommand(s)

Options:
      --completion <SHELL>  Generate the completion code for this shell [possible values:
                            bash, elvish, fish, powershell, zsh]
  -v, --verbose...          Increase logging verbosity
  -q, --quiet...            Decrease logging verbosity
  -h, --help                Print help
  -V, --version             Print version
```

See also the [markdown version of the command line reference](doc/CommandLineHelp.md).

You may also find it convenient to install the completion for your shell. For example, for fish:

```sh
$ yage --completion fish > ~/.config/fish/completions/yage.fish
```

## Usage

First, generate a new age key pair:

```sh
$ yage keygen -o prod.key -p prod.pub
Public key: age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6
```

The public key can be shared with anyone. It allows everybody that has that key to encrypt a secret
that can be decrypted only by someone who has access to the private key. The private key must be
kept secret.

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

Make sure that the private key won't be committed by mistake in the repository, for example, by
adding it to the `.gitignore` file, and by using a tool like
[gitleaks](https://github.com/gitleaks/gitleaks).

```
$ echo "*.key" > .gitignore
```

The private key should be kept in a secure place, for example, in a password manager. It may also
be added to a CI/CD pipeline as
[a secret](https://docs.github.com/actions/security-guides/encrypted-secrets).

Once you have a private and a public key, you can encrypt a YAML file. The `--recipient-file` or
`-R` option is used to specify a file containing the public keys to use for encryption. The
recipients can also be specified directly on the command line with the `--recipient` or `-r`
option.

```sh
$ yage encrypt --recipient-file prod.pub secrets.yaml --output secrets.enc.yaml
```

If you prefer you can encrypt the file in place wit the `--in-place` or `-i` option:

```sh
$ yage encrypt -iR prod.pub secrets.yaml
```

You need the private key to have access to the decrypted values, so if you don't have it, the
encrypted file is showing you what is encrypted, for example `backend.password`, but not the
values.

```yaml
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBRVW9wTXo0dG9lWTRyd1gxdm8yTGNOb1VuQWZSajBETzF4YThMbVVGM21FCjBTQXZxVDIvTWxGd1N6YXlUTHpoVlMzbTVURDZHcXBYaVc5NitYaE1LSW8KLT4gayMtZ3JlYXNlIFwganwhJUcyS1QgTT5sWTMzblYKCi0tLSBLUnREbytjalY3Rm45aEdVVnIzWG8yWC9RUVdlK1A4Mm9BSFdtamg5N2RNCm38HthiQvHqtUIu6+wKOyOH0WShltaeTGk2Qilym+9WFFb0n8g5Eb/6|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
```

But with just the public key, you can still modify the file, for example to add a new secret:

```yaml
mail:
  apiKey: my_secret_key_to_send_emails
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBRVW9wTXo0dG9lWTRyd1gxdm8yTGNOb1VuQWZSajBETzF4YThMbVVGM21FCjBTQXZxVDIvTWxGd1N6YXlUTHpoVlMzbTVURDZHcXBYaVc5NitYaE1LSW8KLT4gayMtZ3JlYXNlIFwganwhJUcyS1QgTT5sWTMzblYKCi0tLSBLUnREbytjalY3Rm45aEdVVnIzWG8yWC9RUVdlK1A4Mm9BSFdtamg5N2RNCm38HthiQvHqtUIu6+wKOyOH0WShltaeTGk2Qilym+9WFFb0n8g5Eb/6|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
```

You can encrypt the new secrets by just using the same command as before:

```sh
$ yage encrypt -iR prod.pub secrets.yaml
```

Or you can omit the recipients, and `yage` will use the recipients from the encrypted file:

```sh
$ yage encrypt -i secrets.yaml
```

`secrets.yaml` now contains the encrypted values:

```yaml
mail:
  apiKey: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBPTmRHcGhPUnJQa2ZjbTVEUEtyT3g1bFd5REdnckF6Z0ZLQzlTekV5THdJCkoybXY3SEI2N3FTcXlHenByRHJOVGFtR2VRUWFBWGhEOGphbkd2ekV3bVkKLT4geUt2b0I2Ly1ncmVhc2UKODVocmxxODlZME1Sa1UvV2RnQkNPcjhvTWpZZFlKYzNkQmsKLS0tIHhsNEpvYzNwT1FMd1c2bmxLQmxOeGZxWnlXbDJUUmVsTklxZVIweUxSQXcK51Wf0RiFIAXYfsbmyMsyQRON5rhQxver8PUU8PDAMIm0XeBSKOzL3ngCmOKGeacahMOY5tWC6DgP20MrtQ==|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
backend:
  password: yage[YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBRVW9wTXo0dG9lWTRyd1gxdm8yTGNOb1VuQWZSajBETzF4YThMbVVGM21FCjBTQXZxVDIvTWxGd1N6YXlUTHpoVlMzbTVURDZHcXBYaVc5NitYaE1LSW8KLT4gayMtZ3JlYXNlIFwganwhJUcyS1QgTT5sWTMzblYKCi0tLSBLUnREbytjalY3Rm45aEdVVnIzWG8yWC9RUVdlK1A4Mm9BSFdtamg5N2RNCm38HthiQvHqtUIu6+wKOyOH0WShltaeTGk2Qilym+9WFFb0n8g5Eb/6|r:age15eesfkh778yljxzgwdq5vaqmmchg5py480vplsymzzqf0dwe5gnqrexdq6]
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

If you're decrypting in a CI/CD pipeline, you may find it convenient to put the private key in the
`YAGE_KEY` environment variable. This way you can just use `yage decrypt -i secrets.yaml`.

You may also find it convenient to pass the private key on the standard input, for example to avoid
storing the private key on disk:

```sh
$ vault-get-key | yage decrypt --key-file - secrets.yaml
```

If you have the private key, you can edit the file in place with your favorite text editor
configured in the `EDITOR` environment variable:

```sh
$ export EDITOR=micro
$ yage edit -K prod.key secrets.yaml
```

The file is edited in clear in the editor and re-encrypted when you save and quit. Here again, only
the modified values are re-encrypted. The others are left unchanged to allow easy tracking of
changes.

Finally, with the private key, you can use the secrets in the encrypted file to run a command with
the environment variables set to the decrypted values in a single command:

```sh
$ yage run -K prod.key secrets.yaml env terraform apply
```

## Pre-commit hook

`yage` can be used in a [pre-commit hook](https://pre-commit.com/) to make sure that the secrets
are always encrypted before committing them to the repository. Here is an example of a
`.pre-commit-config.yaml` file that uses `yage` to detect the non-encrypted secrets in a YAML file
before committing them:

```yaml
repos:
  - repo: https://github.com/glehmann/yage
    rev: 0.5.0
    hooks:
      - id: yage-detect
        files: "secrets-.+\\.yaml"
```

The `files` option is a regular expression that matches the files that should be checked by `yage`.

If your public key is in the repository, you can also add a hook to encrypt the secrets before
committing them:

```yaml
repos:
  - repo: https://github.com/glehmann/yage
    rev: 0.5.0
    hooks:
      - id: yage-encrypt
        files: "secrets-prod-.+\\.yaml"
        args: ["--in-place", "--recipient-file=prod.pub"]
```

`yage-detect` and `yage-encrypt` hooks require the `yage` binary to be installed in the environment
where the hook is running.

The `yage-detect-rust` and `yage-encrypt-rust` hooks are other available variants that build yage
from source.

If you're already using docker in your project, the easiest alternatives are the
`yage-detect-docker` and `yage-encrypt-docker` hooks. They only require docker to be installed in
the environment where the hook is running. The `yage` image is downloaded automatically when the
hook is run for the first time.

## Why?

Mostly to unlock the ability to add values to an encrypted file without having to decrypt it, thing
that is not possible with SOPS. Something I've not been the only one frustrated with, see
[here](https://github.com/getsops/sops/discussions/1081),
[here](https://stackoverflow.com/questions/74103453/is-it-possible-to-update-a-sops-encrypted-file-without-decrypting-it-first),
[here](https://github.com/getsops/sops/issues/1117),
[here](https://github.com/getsops/sops/issues/833), …

And because writing command line tools in rust is fun!

## Still to be done

- [ ] Support comments. Sadly no YAML library that I know of supports comments, so this will be a
      bit tricky.
- [ ] Support age plugins. `age` has a plugin system that could be used to add support for other
      encryption methods.
- [ ] Support multi-document YAML files. This could help to make the CLI more consistent between in
      place and standard output operations.

## License

`yage` is distributed under the terms of the MIT license.

See [LICENSE](LICENSE) for details.
