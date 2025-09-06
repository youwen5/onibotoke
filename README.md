# onibotoke

> 鬼仏 (onibotoke)，lit. "Demonic Buddha"

A no-nonsense fuzzy project search tool. Like
[zoxide](https://github.com/ajeetdsouza/zoxide), but specifically for jumping
to projects. Like `zoxide`, provides a binary called `o`.

Elevator pitch: hit `o <user>/<repo>` to quickly jump to its directory. If it
doesn't exist already, automatically clone it. Type either `<user>` or `<repo>`
partially and it will still work. Optionally specify a remote to choose between
different Git forges.

Named after the onibotoke (Sculptor's Idols) in Sekiro: Shadows Die Twice, that
allow you to fast travel around the map.

![sculptor's idol](https://static.wikia.nocookie.net/shadowsdietwice/images/e/e9/Sculptors_Idol.jpg/revision/latest?cb=20190323220314)

## Installation

Onibotoke is designed for and built into `functorOS`, the functor.systems Linux
distribution. Hence it is deployed using Nix. Run `nix build` in this repo to
obtain `result/lib/onibotoke.nu`, then `source <path-to-onibotoke.nu>`
somewhere in Nushell configuration. In theory, you can manually compile the
binary, then edit `onibotoke.nu` to replace ONIBOTOKE_BIN with the `onibotoke`
binary.

A NixOS module is also provided.

## Detailed usage instructions

Currently, `onibotoke` only works on Nushell. In theory it is easy to add
support for other shells, but some minor refactoring is necessary and I can't
be bothered.

Usage is simple, this is the full CLI interface:
```
o <user>/<repo> <remote>
```

By default, `onibotoke` assumes your source code is stored at `$HOME/Source`.
It will create a directory called `$HOME/Source/by-user` to store projects at,
in the form `$HOME/Source/by-user/<user>/<repo>`. You can change the directory
in [configuration](#configuration).

`<remote>` is optional, and if `<remote>` is not provided, the default will be
used. By default, this is `https://github.com/`. The way remotes are specified
are naive and very simple, it's just the first component of the URL before
`<user>/<repo>` are affixed. That's why the trailing `/` is essential at the
end of that URL. You can easily use arbitrary URLs this way. For example, to
use SSH to clone from GitHub instead, just pass `git@github.com:` for
`<remote>`. Similar to user aliases, you can also add additional remote aliases
as shorthand or change the default remote in the configuration, see
[configuration](#configuration) for more info.

By default, if `<user>` and `<repo>` don't get direct matches, they will be
prefix-matched against the users and repositories already stored in the
projects directory. This is _not_ a fuzzy match, although Smith-Waterman fuzzy
matching may be introduced in a future version. If multiple matches are
obtained, a fuzzy picker will be opened to choose between them (in this case,
the matching will be Smith-Waterman).

You can also set custom aliases for `<user>`. For example, if you alias `me` to
`your-github-username`, then `me/<repo>` will automatically translate to
`your-github-username/<repo>`. See [configuration](#configuration)

A few examples should suffice.

```sh
o functor.systems/functorOS # jumps to $HOME/Source/by-user/functor.systems/functorOS
o func/func # jumps to same location as above, assuming no other users/repos match the pattern
o func/functorOS # same as above
o functor.systems/f # same as above

o quantum9innovation/suntheme # works as expected
o q9i/netsanet # works as expected
o q/suntheme # since q matches both q9i and quantum9innovation, a fuzzy-picker will be opened
```

A few examples of custom remotes.

Suppose I defined the remote `cfs = ssh://forgejo@code.functor.systems/` in my
configuration. Then this will clone from
`ssh://forgejo@code.functor.systems/youwen/onibotoke`, assuming that it doesn't
exist already on my system, of course.

```sh
o youwen/onibotoke cfs
```

I can also clone from an arbitrary remote without defining it in my
configuration beforehand. Notice the trailing `:` symbol.

```sh
o mesa/mesa git@ssh.gitlab.freedesktop.org:
# clones git@ssh.gitlab.freedesktop.org:mesa/mesa.git
```

A few anti-examples, too.

```sh
o f.s/functorOS
```

This does not work as no Smith-Waterman fuzzy match is employed.

```sh
o mesa/mesa https://gitlab.freedesktop.org
```

This does not work as there is no trailing slash, so the URL will be processed
as `https://gitlab.freedesktop.orgmesa/mesa.git`.

## Configuration

`onibotoke` has a few configuration options. Running the tool for the first
time will generate a default configuration. You can also run `o conf --generate-config` to
create one manually. It will be created at `$XDG_CONFIG_HOME/onibotoke/config.toml`.

Sample config:

```toml
projects_dir = "/home/youwen/Source"
default_remote = "gh"

[remote_aliases]
gh = "git@github.com:"
cfs = "ssh://forgejo@code.functor.systems/"

[user_aliases]
me = "youwen"
q9i = "quantum9innovation"
```

By defining `remote_aliases`, you can add other Git sources. You can set
`default_remote` to one of the `remote_aliases` to select the default used when
no remote is specified. Again, these remote URLs are naively specified, when
cloning, `<repo>/<name>` is simply affixed to the end. So make sure to leave in
trailing `/` or `:` when necessary.

## Wishlist

Features I want to implement:

- [ ] Less panics
- [ ] Smith-Waterman fuzzy matching
- [ ] Smarter partial matching on usernames by checking if the corresponding repo exists for the a user first.
    - Prompt the user before cloning a new repository and allow them to fuzzy
      search for a username first. e.g. if I enter `o q/functorOS` and
      `quantum9innovation` and `q9i` are both users, it should prompt before
      cloning with `quantum9innovation` and `q9i` as options
