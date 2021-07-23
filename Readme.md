# Git Changelog

An application for generating a changelog from git commit tree

<https://git.claudiomattera.it/claudiomattera/git-changelog>


Installation
----

Executables can be downloaded in the [releases page](./releases).

If the executable `git-changelog` is put inside a directory included in the `PATH` environment variable, a new git subcommand `changelog` is available.

~~~~shell
git changelog /path/to/git/repository > changelog.txt
~~~~


### Installation from Source

This is a Rust application and can be installed using Cargo.

~~~~shell
cargo build --release

# Resulting executable is in ./target/release/git-changelog
~~~~


Usage
----

This application makes the following assumptions about the git commit tree.

* Version names follow [Semantic Versioning] format;
* Available versions are encoded in git tags (tags not matching the format are ignored);
* Versions are developed in sequence;
* Changes are encoded in commits following a certain format (commit not matching such format are ignored).

The application looks at the commit tree and considers all changes between each pair of subsequent versions.
then it creates a Markdown changelog listing all changes for each version and prints it to standard output.

~~~~plain
git-changelog 0.1.0
Claudio Giovanni Mattera <dev@claudiomattera.it>
Generate changelog from git commit tree

USAGE:
    git-changelog [FLAGS] [OPTIONS] <repo-path>

FLAGS:
        --add-tag-description    Add version description from tag messages
    -h, --help                   Prints help information
    -o, --only-last              Only last version changes
        --strip-gpg-signature    Strip GPG signature from version descriptions
    -V, --version                Prints version information
    -v, --verbose                Verbosity

OPTIONS:
    -c, --commit-regex <commit-regex>
            Commit message regular expression [default: (.+)\s+\(issue\s+#(\d+)\)]

    -r, --commit-replacement <commit-replacement>    Commit message replacement text [default: ${1} (issue ${2})]
    -d, --head-description <head-description>        Set the current head description
    -i, --include-head <include-head>                Include the current head as last version
    -s, --select-version <selected-versions>...      Generate changelog for selected versions

ARGS:
    <repo-path>    Repository path
~~~~

[Semantic Versioning]: https://semver.org/


### Example

Consider the following commit log.

~~~~plain
*   e0f2b3a (HEAD -> master, tag: 0.2.0) Merge branch 'v0.2.0-devel'
|\
| * d65a363 (v0.2.0-devel) Add file d (issue #4)
| | * b808f2d (issue/4) Add file d
| |/
| * 54dde1f Add file c (issue #3)
| | * 8c0a87b (issue/3) Add file c
| |/
| * 6ba5ad1 Start version 0.2.0
|/
*   742c7b3 (tag: 0.1.0) Merge branch 'v0.1.0-devel'
|\
| * 5053c28 (v0.1.0-devel) Add other file (issue #2)
| * a26d0e0 Add file (issue #1)
| | * 0ff918f (issue/2) Add file b
| |/
| | * 63cc891 (issue/1) Add file
| | * 1289020 Other change
| | * 5c7d7e5 Some change
| |/
| * b97af0f Start version 0.1.0
|/
* cc5c841 Initial commit
~~~~

This application will generate the following Markdown changelog.

~~~~markdown
# Version 0.2.0 (2021-07-07)

- Add file d (issue #4)
- Add file c (issue #3)

# Version 0.1.0 (2021-07-07)

- Add other file (issue #2)
- Add file (issue #1)
~~~~


### Select and Convert Commit Messages

Commit messages are selected for changelog according to a regular expression passed via the command-line argument `--commit-regex`, and converted by substituting the pattern passed via `--commit-replacement`.
Default values are, respectively, `(.+)\s+\(issue\s+#(\d+)\)` and `${1} (issue ${2})`.

In order to include links to the repository issues in the changelog, use something like:

~~~~plain
${1} (issue [${2}](https://git.claudiomattera.it/claudiomattera/git-changelog/issues/#{2}))`
~~~~


### Include Current Head

It is possible to generate a changelog for the current head, so that it can be included in the upcoming git tag.

In the following example, the version `1.0.3` is assigned to the current head.

~~~~shell
git-changelog --include-head 1.0.3
~~~~


### Include Tag Descriptions in Changelog

The tag message can be used as a summary before the list of changes in the changelog.

~~~~shell
git-changelog --add-tag-description
~~~~

This application will generate the following Markdown changelog.

~~~~markdown
# Version 0.2.0 (2021-07-07)

This is the text in the tag message.

It can contain arbitrary text, it is not parsed as Markdown, but copied verbatim.

- Add file d (issue #4)
- Add file c (issue #3)

# Version 0.1.0 (2021-07-07)

- Add other file (issue #2)
- Add file (issue #1)
~~~~

In case tags are signed, the flag `--strip-gpg-signature` will strip the signature in the changelog.


License
----

Copyright Claudio Mattera 2021

You are free to copy, modify, and distribute this application with attribution under the terms of the [MPL 2.0 license]. See the [`License.txt`](./License.txt) file for details.

[MPL 2.0 license]: https://opensource.org/licenses/MPL-2.0
