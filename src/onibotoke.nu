export def __onibotoke [user_repo: string] {
  let x = $user_repo | split row '/'
  let user = $x.0
  let repo = $x.1

  let conf = read_config

  ensure_paths_exist $conf

  let user_search_result = find_matching_users $conf --user $user --repo $repo
  if not $user_search_result.found {
	return [$conf.projects_dir "by-user" $user $repo] | path join | into string
  }

  let selected_user = $user_search_result.0

  let repo_search_result = find_matching_repos $conf --user $user --repo $repo
  if not $repo_search_result.found {
	return [$conf.projects_dir "by-user" $user $repo] | path join | into string
  }

  let selected_repo = $repo_search_result.0

  return [$conf.projects_dir "by-user" $selected_user $selected_repo] | path join | into string
}

def read_config []: nothing -> record {
  let default_config = {
	projects_dir: $"($env.HOME)/Source"
	default_remote: "github"
	remote_aliases: {
	  github: "git@github.com:"
	}
	user_aliases: { }
	repo_aliases: { }
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

def ensure_paths_exist [config: record] {
  mkdir ($config.projects_dir | path join "by-user")
}

def find_matching_users [config: record, --user: string, --repo: string] {
  let users_dir = $config.projects_dir | path join "by-user"

  let matching_users: table = ls $users_dir | find --columns [name] $user
  if ($matching_users | is-empty) {
	print "I couldn't find any matching users. Clone from a remote? (y/n)"
	let choice = input -d "n" -n 1
	if choice == "y" {
	  clone_repo $config --remote-prefix "git@github.com:" --owner $user --repo $repo
	} else {
	  return {
		found: false
	  }
	}
  }

  return {
	found: true
	users: ($matching_users | select name)
  }
}

def find_matching_repos [config: record, --user: string, --repo: string] {
  let repos_dir = $config.projects_dir | path join "by-user" | path join $user

  let matching_repos: table = ls $repos_dir | find --columns [name] $repo
  if ($matching_repos | is-empty) {
	print "I couldn't find any matching repos for that user. Clone from a remote? (y/n)"
	let choice = input -d "n" -n 1
	if choice == "y" {
	  clone_repo $config --remote-prefix "git@github.com:" --owner $user --repo $repo
	}
	return {
	  found: false
	}
  }

  return {
	found: true
	users: ($matching_repos | select name)
  }
}

def clone_repo [config: record, --remote-prefix: string, --owner: string, --repo: string] {
  let git_clone = git clone ($"($remote_prefix)($owner)/($repo)") ([$config.projects_dir "by-user" $owner $repo] | path join | into string) | complete
  if $git_clone.exit_code != 0 {
	print "Something went wrong trying to clone from remote."
	exit $git_clone.exit_code
  }
}
