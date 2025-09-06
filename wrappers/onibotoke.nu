def --env o [owner_repo] {
  let s = $owner_repo | split row '/'
  let owner = $s | get 0
  let repo = $s | get 1

  cd (cargo run -- --owner $owner --repo $repo --forge-url "git@github.com:")
}
