# Command-Line Help for `yage`

This document contains the help content for the `yage` command-line program.

**Command Overview:**

* [`yage`↴](#yage)
* [`yage check`↴](#yage-check)
* [`yage decrypt`↴](#yage-decrypt)
* [`yage edit`↴](#yage-edit)
* [`yage encrypt`↴](#yage-encrypt)
* [`yage env`↴](#yage-env)
* [`yage keygen`↴](#yage-keygen)
* [`yage pubkey`↴](#yage-pubkey)

## `yage`

A simple tool to manage encrypted secrets in YAML files with age encryption

**Usage:** `yage [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `check` — Check the encryption status of a YAML file
* `decrypt` — Decrypt the values in a YAML file
* `edit` — Edit an encrypted YAML file
* `encrypt` — Encrypt the values in a YAML file
* `env` — Execute a command with the environment from the encrypted YAML file
* `keygen` — Generate a new age key
* `pubkey` — Convert private age keys to their public key

###### **Options:**

* `--completion <SHELL>` — Generate the completion code for this shell

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `yage check`

Check the encryption status of a YAML file

**Usage:** `yage check [FILES]...`

###### **Arguments:**

* `<FILES>` — The YAML files to decrypt



## `yage decrypt`

Decrypt the values in a YAML file

**Usage:** `yage decrypt [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>` — The YAML files to decrypt

###### **Options:**

* `-i`, `--in-place` — Decrypt in place

  Possible values: `true`, `false`

* `-k`, `--key <KEY>` — Decrypt with the specified key
* `-K`, `--key-file <KEY_FILE>` — Decrypt with the key in the file
* `-o`, `--output <OUTPUT>` — The output path to the decrypted YAML file

  Default value: `-`



## `yage edit`

Edit an encrypted YAML file

The file is decrypted with the specified keys and open in a text editor. The user can edit the file and save it. The values are then encrypted with the same keys and the recipients, and saved in the original file.

The YAML file may contain some unencrypted values, and some encrypted values. The encrypted values are decrypted before the edition and all the values are encrypted after the edition.

Only the modified values are encrypted, the other values are left unchanged.

**Usage:** `yage edit [OPTIONS] --editor <EDITOR> <FILE>`

###### **Arguments:**

* `<FILE>` — The encrypted YAML file to edit

###### **Options:**

* `-e`, `--editor <EDITOR>` — The editor command to use
* `-k`, `--key <KEY>` — Decrypt with the specified key
* `-K`, `--key-file <KEY_FILE>` — Decrypt with the key at in this file
* `-r`, `--recipient <RECIPIENT>` — Encrypt to the specified recipients
* `-R`, `--recipient-file <RECIPIENT_FILE>` — Encrypt to recipients listed at PATH



## `yage encrypt`

Encrypt the values in a YAML file

Only the values are encrypted, the keys are left in clear.

The values are encrypted with the recipients' public keys in the age format, converted in base64 and surrounded by `yage[…]` markers.

This command is able to encrypt some new values in a file that already contains encrypted values. The encrypted values are detected thanks to the `yage[…]` markers and left unchanged.

**Usage:** `yage encrypt [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>` — The YAML files to encrypt

###### **Options:**

* `-i`, `--in-place` — Encrypt in place

  Possible values: `true`, `false`

* `-r`, `--recipient <RECIPIENT>` — Encrypt to the specified recipients
* `-R`, `--recipient-file <RECIPIENT_FILE>` — Encrypt to recipients listed at PATH
* `-o`, `--output <OUTPUT>` — The output path to the encrypted YAML file

  Default value: `-`



## `yage env`

Execute a command with the environment from the encrypted YAML file

The YAML file must contain a map with string keys and values. The keys are the environment variable names, and the values are the environment variable values. Other more complex YAML structures are not supported.

**Usage:** `yage env [OPTIONS] <FILE> <COMMAND> [ARGS]...`

###### **Arguments:**

* `<FILE>` — The YAML file to decrypt
* `<COMMAND>` — The command to run
* `<ARGS>` — The command arguments

###### **Options:**

* `-i`, `--ignore-environment` — Start with an empty environment

  Default value: `false`

  Possible values: `true`, `false`

* `-k`, `--key <KEY>` — Decrypt with the specified key
* `-K`, `--key-file <KEY_FILE>` — Decrypt with the key at PATH



## `yage keygen`

Generate a new age key

The public part of the key is logged to the standard error output. It may be computed from the private key with the pubkey command.

The key is written in the age format, which is compatible with the age tool.

**Usage:** `yage keygen [OPTIONS]`

###### **Options:**

* `-o`, `--output <OUTPUT>` — The output path to the private key file

  Default value: `-`
* `-p`, `--public <PUBLIC>` — The output path to the public key file



## `yage pubkey`

Convert private age keys to their public key

The input key and output public key are in the age format, which is compatible with the age tool.

**Usage:** `yage pubkey [OPTIONS] [KEY_FILE]...`

###### **Arguments:**

* `<KEY_FILE>` — The private key files

###### **Options:**

* `-k`, `--key <KEY>` — The private keys
* `-o`, `--output <OUTPUT>` — The output path to the public key file

  Default value: `-`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
