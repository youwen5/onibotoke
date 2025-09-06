def --env o [owner_repo remote?] {
  let s = $owner_repo | split row '/'
  let owner = $s | get 0
  let repo = $s | get 1

  let conf = read_config

  let $owner_resolved = if ($conf.user_aliases? | get $owner --optional) != null {
    $conf.user_aliases | get $owner
  } else {
    $owner
  }

  let $remote_url = match ($remote) {
    null => ($conf.remote_aliases | get $conf.default_remote)
    _ => {
      match ($conf.remote_aliases | get $remote --optional) {
        null => $remote
        _ => ($conf.remote_aliases | get $remote)
      }
    }
  }

  cd (cargo run -- --owner $owner_resolved --repo $repo --forge-url $remote_url)
}

def read_config []: nothing -> record {
  let default_config = {
	projects_dir: $"($env.HOME)/Source"
	default_remote: "gh"
	remote_aliases: {
	  gh: "git@github.com:"
	}
	user_aliases: { }
  }

  let config_base = if ($env.XDG_CONFIG_HOME? == null) {
	$"($env.HOME)/.config"
  } else {
	$env.XDG_CONFIG_HOME
  } | path parse
  let config_dir = $config_base | path join "onibotoke"
  mkdir $config_dir
  let config_file = $config_dir | path join "config.toml"
  if not ($config_file | path exists) {
	$default_config | save $config_file
  }
  let parsed_config: table = open $config_file
  let parsed_config = $parsed_config | update projects_dir {|row| ($row.projects_dir | path parse)}
  return $parsed_config
}
