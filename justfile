set shell := ["nu", "-c"]

msi_directory := "target/release/bundle/msi"
github_pat_token_path := "~/.github/scut_pat.token"
scut_signing_key_path := "~/.tauri/scut.key"
updater_gist_id := "27d0e797ea86132427cbc7674298e612"

dev:
    cargo tauri dev

test:
    cargo test

set-version version:
    cargo set-version {{version}} -p scut -p scut_core -p scut-ui
    
    open scut-ui/src-tauri/tauri.conf.json | \
    update package.version {{version}} | \
    to json | jq | \
    save -f scut-ui/src-tauri/tauri.conf.json

# sets cargo version, commits and releases scut - git tag message -> release notes
release version: test
    echo "releasing version {{version}}"

    just set-version {{version}}
    
    git add .
    git commit -m "v{{version}}"
    git tag -a "v{{version}}"
    git push --tag

    just release-build

    just github-release {{version}}

    just release-tauri-updater {{version}}

release-build:
    $env.TAURI_PRIVATE_KEY = (open {{scut_signing_key_path}}); \
    cargo tauri build

    cargo build --release

# push release to github and uploads assets - requires gh cli
github-release version:
    $env.GH_TOKEN = (open {{github_pat_token_path}}); \
    gh release create \
      --verify-tag v{{version}} \
      --notes-from-tag \
      --draft \
      --title {{version}} \
      {{msi_directory}}/SCUT_{{version}}_x64_en-US.msi.zip \
      {{msi_directory}}/SCUT_{{version}}_x64_en-US.msi.zip.sig

# set the version-info gist so tauri auto-update works
release-tauri-updater version:
    # extract notes from tag message
    let $notes = (git cat-file -p (git rev-parse (git tag -l v{{version}})) | split row "\n" | skip 5 | str join "\\n") ; \
    let $sig = open --raw {{msi_directory}}/SCUT_{{version}}_x64_en-US.msi.zip.sig ; \
    let $url = "https://github.com/drmason13/scut/releases/download/v{{version}}/SCUT_{{version}}_x64_en-US.msi.zip" ; \
      open version-info.json | \
      update notes $"SCUT version (echo $notes | str substring 1..)" | \
      update platforms.windows-x86_64.signature $sig | \
      update platforms.windows-x86_64.url $url | \
      update version "v{{version}}" | \
      to json | jq | save -f version-info.json

    $env.GH_TOKEN = (open {{github_pat_token_path}}); \
    gh gist edit {{updater_gist_id}} version-info.json \
      --filename version-info.json
