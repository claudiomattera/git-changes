// Copyright Claudio Mattera 2021.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use tracing::*;

use git2::{Oid, Repository};

use structopt::StructOpt;

use anyhow::anyhow;
use anyhow::Result;

use semver::Version;

use chrono::{NaiveDate, TimeZone, Utc};

use markdown_composer::{List, Markdown};

use regex::Regex;

mod commandline;
use commandline::Arguments;

mod logging;
use logging::setup_logging;

type DescribedVersion = (Version, Option<String>, Oid);
type DatedVersion = (Version, Option<String>, NaiveDate, Oid);
type VersionChangelog = (Version, Option<String>, NaiveDate, Vec<String>);

fn main() -> Result<()> {
    let arguments = Arguments::from_args();
    setup_logging(arguments.verbosity);

    let commit_regex = arguments.commit_regex.clone();
    let commit_replacement = arguments.commit_replacement.clone();

    let repo = Repository::open(arguments.repo_path)?;
    let versions = find_all_versions(
        &repo,
        arguments.include_head,
        arguments.head_description,
        arguments.strip_gpg_signature,
    )?;
    info!("Found {} versions", versions.len());

    let versions = find_version_dates(&repo, versions)?;
    let version_pairs = pair_versions(versions);
    let version_pairs = filter_versions(version_pairs, arguments.selected_versions);
    let version_pairs = keep_only_last_version(version_pairs, arguments.only_last);
    let changelog = generate_changelog(&repo, version_pairs, |text| {
        process_commit_message(text, &commit_regex, &commit_replacement)
    })?;
    let rendered = render_changelog(changelog, arguments.add_tag_description)?;

    println!("{}", rendered);

    Ok(())
}

fn find_all_versions(
    repo: &Repository,
    head_version: Option<Version>,
    head_description: Option<String>,
    strip_gpg_signature: bool,
) -> Result<Vec<DescribedVersion>> {
    let mut versions = Vec::new();

    let signature_regex =
        Regex::new(r"\n-----BEGIN PGP SIGNATURE-----(?s:.+)-----END PGP SIGNATURE-----").unwrap();

    repo.tag_foreach(|oid, name| {
        debug!("Found tag {}", String::from_utf8_lossy(name));
        if let Ok((version, oid)) = process_tag(oid, name) {
            debug!("Found version {} ({})", version, oid);
            let tag = repo.find_tag(oid).unwrap();
            let mut description = tag.message().map(|s| s.to_owned());
            if strip_gpg_signature {
                description = description
                    .map(|description| signature_regex.replace(&description, "").to_string());
            }
            versions.push((version, description, oid));
        }
        true
    })?;

    let (initial_version, initial_oid) = find_initial_commit(repo)?;
    versions.push((initial_version, Some("".to_owned()), initial_oid));

    if let Some(head_version) = head_version {
        let (head_version, head_oid) = find_head(repo, head_version)?;
        versions.push((head_version, head_description, head_oid));
    }

    versions.sort_by(|(a, ..), (b, ..)| a.cmp(b));
    versions.reverse();

    Ok(versions)
}

fn find_initial_commit(repo: &Repository) -> Result<(Version, Oid)> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    if let Some(oid) = revwalk.into_iter().last() {
        let oid = oid?;
        debug!("Found initial commit {}", oid);
        Ok((Version::parse("0.0.0").unwrap(), oid))
    } else {
        Err(anyhow!("Missing initial commit"))
    }
}

fn find_head(repo: &Repository, version: Version) -> Result<(Version, Oid)> {
    let oid = repo.head()?.peel_to_commit()?.id();
    Ok((version, oid))
}

fn find_version_dates(
    repo: &Repository,
    versions: Vec<DescribedVersion>,
) -> Result<Vec<DatedVersion>> {
    let versions = versions
        .into_iter()
        .map(|(version, description, oid)| {
            let commit = repo.find_object(oid, None)?.peel_to_commit()?;
            let instant = Utc.timestamp(commit.time().seconds(), 0);
            let date = instant.naive_local().date();
            Ok((version, description, date, oid))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(versions)
}

fn pair_versions(versions: Vec<DatedVersion>) -> Vec<(DatedVersion, DatedVersion)> {
    versions
        .clone()
        .into_iter()
        .zip(versions.into_iter().skip(1))
        .collect::<Vec<_>>()
}

fn filter_versions(
    version_pairs: Vec<(DatedVersion, DatedVersion)>,
    selected_versions: Option<Vec<Version>>,
) -> Vec<(DatedVersion, DatedVersion)> {
    if let Some(selected_versions) = selected_versions {
        debug!("Filtering only selected versions");
        trace!("Selected versions: {:?}", selected_versions);
        version_pairs
            .into_iter()
            .filter(|((first_version, ..), _)| selected_versions.contains(first_version))
            .collect()
    } else {
        version_pairs
    }
}

fn keep_only_last_version(
    version_pairs: Vec<(DatedVersion, DatedVersion)>,
    only_last: bool,
) -> Vec<(DatedVersion, DatedVersion)> {
    if only_last {
        info!("Keeping only last version");
        version_pairs.into_iter().take(1).collect()
    } else {
        version_pairs
    }
}

fn generate_changelog<F>(
    repo: &Repository,
    version_pairs: Vec<(DatedVersion, DatedVersion)>,
    commit_processor: F,
) -> Result<Vec<VersionChangelog>>
where
    F: Fn(&str) -> Option<String> + Clone,
{
    let changelog = version_pairs
        .into_iter()
        .map(
            |((version, description, date, oid), (previous_version, _, _, previous_oid))| {
                info!(
                    "Generating changelog between {} and {}",
                    previous_version, version,
                );

                let mut revwalk = repo.revwalk()?;
                let range = format!("{}..{}", previous_oid, oid);
                debug!("Range: {}", range);
                revwalk.push_range(&range)?;

                let version_changelog = generate_version_changelog(
                    &repo,
                    &previous_oid,
                    &oid,
                    commit_processor.clone(),
                )?;

                debug!(
                    "Found {} changelog entries for version {}",
                    version_changelog.len(),
                    version,
                );
                Ok((version, description, date, version_changelog))
            },
        )
        .collect::<Result<Vec<_>>>()?;

    Ok(changelog)
}

fn render_changelog(changelog: Vec<VersionChangelog>, add_tag_description: bool) -> Result<String> {
    let mut output = String::new();

    for (version, description, date, changelog) in changelog {
        let mut markdown = Markdown::new();
        markdown.header1(format!("Version {} ({})", version, date));
        output.push_str(&markdown.render());

        let mut markdown = Markdown::new();
        if add_tag_description {
            if let Some(description) = description {
                // Description is already in Markdown format
                output.push_str(&description.trim());
                output.push('\n');
                output.push('\n');
                markdown.header2("Changes");
            }
        }

        let mut list_builder = List::builder();
        for entry in changelog {
            list_builder = list_builder.append(entry.clone());
        }
        markdown.list(list_builder.unordered());
        output.push_str(&markdown.render());
    }

    Ok(output)
}

fn process_tag(oid: Oid, name: &[u8]) -> Result<(Version, Oid)> {
    let name = String::from_utf8(name.to_vec())?;
    trace!("Name as String: {}", name);
    let name = name
        .strip_prefix("refs/tags/")
        .ok_or_else(|| anyhow!("Not a tag ref"))?;
    trace!("Name without prefix: {}", name);
    let version = Version::parse(&name)?;
    trace!("Version: {}", version);
    Ok((version, oid))
}

fn process_commit_message(text: &str, regex: &Regex, replacement: &str) -> Option<String> {
    trace!("Commit message: {}", text);

    if let Some(captures) = regex.captures(text) {
        trace!("Commit message matches regex");
        let mut changelog_entry = String::new();
        captures.expand(replacement, &mut changelog_entry);
        trace!("Commit message expanded to: {}", changelog_entry);

        Some(changelog_entry)
    } else {
        None
    }
}

fn generate_version_changelog<F>(
    repo: &Repository,
    previous_oid: &Oid,
    oid: &Oid,
    commit_processor: F,
) -> Result<Vec<String>>
where
    F: Fn(&str) -> Option<String>,
{
    let mut revwalk = repo.revwalk()?;
    let range = format!("{}..{}", previous_oid, oid);
    debug!("Range: {}", range);
    revwalk.push_range(&range)?;

    let changelog = revwalk
        .into_iter()
        .map(|oid| {
            let oid: Oid = oid?;
            let commit = repo.find_commit(oid)?;
            let changelog_entry = commit.summary().and_then(|text| commit_processor(text));

            if let Some(changelog_entry) = changelog_entry {
                let changelog_entry: String = changelog_entry;
                debug!("Found changelog entry {}", changelog_entry);
                Ok(Some(changelog_entry))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let changelog = changelog.into_iter().flatten().collect();

    Ok(changelog)
}
