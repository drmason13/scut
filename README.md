# SCUT

> Strategic Command Utility Tool

A program to upload and download SC saves to/from remote shares like dropbox.

## User Guide

### Installation

Simply download from [GitHub Releases](https://github.com/drmason13/scut/releases)

### Configuration

### Usage

Just run `scut.exe` to download or upload saves. It will try and figure out what to do by itself.

Or you can run `scut.exe --background` to start a desktop tray app on Windows.
Click to open a small window to see what scut has planned and hit the big button to go do it.

## Developer Guide

## Release

> this is for developers looking to publish a new version of scut

To ship a new version of scut run the justfile!

```sh
just release
```

> NOTE: scut's justfile uses [nushell](https://www.nushell.sh/book/installation.html) to run commands, you can install it on windows and linux

This will...
  * update the package.version in scut* Cargo.toml files
  * add a commit to git with the version as the message
  * tag that commit

you will be prompted for a git tag message which is used to create release notes:

example tag message:
```
v0.4.4

* automate release process
* documentation for users and developers in github README.md
```

  * build the tauri app (*you will need to enter the password to sign it using your key*)
  * upload it to [GitHub Releases](https://github.com/drmason13/scut/releases) using the gh CLI (*requires a GitHub Personal Access Token at `~/.github/scut_pat.token`*)
  * update the url in the [JSON hosted as GitHub gist](https://gist.github.com/drmason13/27d0e797ea86132427cbc7674298e612/edit)
