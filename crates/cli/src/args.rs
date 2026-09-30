//! CLI grammar, validation and the CLI → core mapping (spec §7).
//!
//! # Layers
//!
//! 1. clap parses the grammar (`-h` / `-V`, positional shapes, required
//!    arguments, `u64` overflow for `--edit` / `--reply-to` / `--comment`)
//!    and returns grammar errors to main for one-line stderr and exit 2;
//!    explicit help/version keep clap's stdout and exit 0. `guide` is a
//!    typed information command on [`TopCommand`], flattened away from the
//!    fourteen business [`Commands`].
//! 2. [`resolve`] validates every business value **before any file access**
//!    (spec §7.1.5–§7.1.7): nick / status / section grammars, control
//!    characters and double spaces in the enumerating values, `-m`
//!    normalization, `--since` / `--for` resolution against the one `Ctx`
//!    clock, `--regex` compilation, the six brief forms, profile pairing,
//!    the two state forms, and walk glob validity. Every failure is exactly
//!    one stderr line `error: <msg>` and exit 2. `guide` never enters
//!    [`resolve`].
//! 3. The result is a plain-value [`coexist_core::ops::Request`] (or a wait
//!    invocation): no clap type crosses into core.
//!
//! Paths carry `consts::SUFFIX` appended when the user omitted it
//! (spec §2.4.3); `--target` goes through `hash::normalized_target` and
//! `-m` bodies through `names::normalize_body` before reaching core.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use coexist_core::consts;
use coexist_core::doc;
use coexist_core::error::Fault;
use coexist_core::fsx;
use coexist_core::hash;
use coexist_core::names;
use coexist_core::ops::context::Ctx;
use coexist_core::ops::{
    BriefForm, BriefParams, BriefTarget, CheckParams, CheckStateParams, Filters, ListParams,
    LookforCommentParams, PostParams, ProfileAction, ProfileParams, ReadBriefParams,
    ReadNotificationParams, RegexOpt, Request, StateDeclare, StateParams, StateRemove,
    SubscribeParams,
};
use coexist_core::time;
use coexist_core::wait;

/// One dispatchable invocation produced by [`resolve`].
pub enum Invocation {
    /// A base command: the validated [`Request`] plus the object its report
    /// first line and diag context are built from.
    Base {
        command: &'static str,
        kind: DiagKind,
        request: Request,
    },
    /// A wait extension command, dispatched by `main` only — base commands
    /// never reference `coexist_core::wait`. The report command/object (and
    /// the stderr ready line) are derived in `main` from
    /// `wait::command_object(&path, &filter)` so they can never disagree.
    Wait {
        kind: DiagKind,
        path: PathBuf,
        filter: wait::Filter,
    },
}

/// The object a command addresses: the managed file (suffix already
/// appended by [`suffixed`]) or the directory as given (`.` when omitted).
#[derive(Debug)]
pub enum DiagKind {
    File(PathBuf),
    Dir(PathBuf),
}

/// The parsed command line: information `guide` plus the fourteen business
/// commands (eleven base + three wait extensions).
#[derive(Parser, Debug)]
#[command(
    name = "agent-coexist",
    version,
    about = "Local markdown collaboration files for agents: posts, briefs, agent-profiles and directory state",
    after_help = TOP_AFTER_HELP,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: TopCommand,
}

/// Top-level dispatch: `guide`, or one of the flattened business [`Commands`].
#[derive(Subcommand, Debug)]
pub enum TopCommand {
    /// Print an offline operational guide page
    #[command(override_usage = U_GUIDE, after_help = H_GUIDE)]
    Guide {
        /// start, read, write or wait; omit for the index page
        #[arg(value_name = "start|read|write|wait", value_parser = parse_guide_topic)]
        topic: Option<GuideTopic>,
    },
    #[command(flatten)]
    Business(Commands),
}

/// Exact lowercase guide topic; omitted topic selects the default page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideTopic {
    Start,
    Read,
    Write,
    Wait,
}

fn parse_guide_topic(raw: &str) -> Result<GuideTopic, String> {
    match raw {
        "start" => Ok(GuideTopic::Start),
        "read" => Ok(GuideTopic::Read),
        "write" => Ok(GuideTopic::Write),
        "wait" => Ok(GuideTopic::Wait),
        _ => Err(format!(
            "'{}' is not a valid guide topic; expected start, read, write or wait",
            names::fold(raw)
        )),
    }
}

/// The `WALK` argument group shared by `list`, `read-notification` and
/// `check-state` (spec §7.2).
#[derive(Args, Debug)]
pub struct WalkArgs {
    /// only files matching this gitignore-style glob, relative to DIR (repeatable; cannot reopen ignore-pruned directories, pair with --no-ignore)
    #[arg(long, value_name = "glob")]
    pub include: Vec<String>,
    /// skip files matching this glob, relative to DIR (repeatable)
    #[arg(long, value_name = "glob")]
    pub exclude: Vec<String>,
    /// apply no ignore rules (.gitignore, .ignore, .git/info/exclude, global gitignore)
    #[arg(long)]
    pub no_ignore: bool,
}

/// `--type` value (spec §7.1.8): the four type words; `profile` is accepted
/// as an alias for `agent-profile`.
#[derive(Clone, Copy, Debug)]
pub(crate) enum KindArg {
    Post,
    Brief,
    Profile,
    State,
}

fn parse_type(raw: &str) -> Result<KindArg, String> {
    match raw {
        "post" => Ok(KindArg::Post),
        "brief" => Ok(KindArg::Brief),
        "agent-profile" | "profile" => Ok(KindArg::Profile),
        "state" => Ok(KindArg::State),
        _ => Err(format!(
            "'{}' is not a valid type; expected post, brief, agent-profile (alias: profile) or state",
            names::fold(raw)
        )),
    }
}

fn kind_of(kind: KindArg) -> doc::Kind {
    match kind {
        KindArg::Post => doc::Kind::Post,
        KindArg::Brief => doc::Kind::Brief,
        KindArg::Profile => doc::Kind::Profile,
        KindArg::State => doc::Kind::State,
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Recursively list collaboration files under DIR
    #[command(override_usage = U_LIST, after_help = H_LIST)]
    List {
        /// directory to scan (default: the current directory; the scan is recursive)
        dir: Option<PathBuf>,
        /// only these file types; repeatable, union (post, brief, agent-profile, state; profile is an alias)
        #[arg(long = "type", value_name = "type", value_parser = parse_type)]
        types: Vec<KindArg>,
        #[command(flatten)]
        walk: WalkArgs,
    },
    /// Verify one file and report status without target content
    #[command(override_usage = U_CHECK, after_help = H_CHECK)]
    Check {
        /// the collaboration file; the '.agent-besties.md' suffix may be omitted
        path: PathBuf,
    },
    /// Create or update an agent profile
    #[command(override_usage = U_PROFILE, after_help = H_PROFILE)]
    Profile {
        /// the agent-profile file; the suffix may be omitted
        path: PathBuf,
        /// your nick; used only for the functional-lock check, never written
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the section to set or remove
        #[arg(long, value_name = "name")]
        section: Option<String>,
        /// for this command: the body of --section, stored verbatim in a code fence
        #[arg(short = 'm', value_name = "content")]
        message: Option<String>,
        /// delete the whole --section
        #[arg(long)]
        remove: bool,
    },
    /// Append a comment, or edit your own
    #[command(override_usage = U_POST, after_help = H_POST)]
    Post {
        /// the post file; the suffix may be omitted
        path: PathBuf,
        /// the author nick recorded in the comment header
        #[arg(long = "as", value_name = "nick")]
        as_nick: String,
        /// for this command: the comment body, stored verbatim in a code fence
        #[arg(short = 'm', value_name = "content")]
        message: String,
        /// rewrite your own comment #n (its author must match --as)
        #[arg(long, value_name = "n")]
        edit: Option<u64>,
    },
    /// Join or leave a post's subscriber list
    #[command(override_usage = U_SUBSCRIBE, after_help = H_SUBSCRIBE)]
    Subscribe {
        /// the post file; the suffix may be omitted
        path: PathBuf,
        /// the nick added to (or, with --cancel, removed from) the list
        #[arg(long = "as", value_name = "nick")]
        as_nick: String,
        /// remove --as from the subscriber list instead of adding it
        #[arg(long)]
        cancel: bool,
    },
    /// Create or update a brief and its entries
    #[command(override_usage = U_BRIEF, after_help = H_BRIEF)]
    Brief {
        /// the brief file; the suffix may be omitted
        path: PathBuf,
        /// your nick; functional-lock check only, never written
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the description (no --target/--entry) or the entry's note (with --target/--entry)
        #[arg(short = 'm', value_name = "content")]
        message: Option<String>,
        /// the entry's target file, relative to the brief's directory; raw CR and LF are rejected
        #[arg(long, value_name = "path")]
        target: Option<String>,
        /// the entry title (matched after whitespace folding)
        #[arg(long, value_name = "title")]
        entry: Option<String>,
        /// pattern checked when the entry is read (no match makes it stale; writing only checks that the pattern compiles); '' deletes every regex line of the entry; only with --target; raw CR and LF are rejected
        #[arg(long, value_name = "pattern")]
        regex: Option<String>,
        /// delete the whole --entry
        #[arg(long)]
        remove: bool,
        /// recompute '- hash:' lines from current target bytes
        #[arg(long)]
        refresh: bool,
    },
    /// Declare or remove directory state (locks, observations)
    #[command(override_usage = U_STATE, after_help = H_STATE)]
    State {
        /// the directory (default: the current directory); must already exist
        dir: Option<PathBuf>,
        /// the declaring nick
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the status name; required in the declare form; in the remove form, a filter (omitted: every status)
        #[arg(long, value_name = "name")]
        status: Option<String>,
        /// duration of the declaration (declare form only)
        #[arg(long = "for", value_name = "duration")]
        for_duration: Option<String>,
        /// switch to the remove form: remove this nick's entries
        #[arg(long, value_name = "nick")]
        remove: Option<String>,
    },
    /// List mentions, replies, subscriptions and state changes since a time
    #[command(override_usage = U_READ_NOTIFICATION, after_help = H_READ_NOTIFICATION)]
    ReadNotification {
        /// directory to scan (default: the current directory; recursive, plus ancestor state files)
        dir: Option<PathBuf>,
        /// your nick
        #[arg(long = "as", value_name = "nick")]
        as_nick: String,
        /// RFC 3339 time or a duration back from now
        #[arg(long, value_name = "time|duration")]
        since: String,
        #[command(flatten)]
        walk: WalkArgs,
    },
    /// Find comments by keyword, author, mention or number
    #[command(override_usage = U_LOOKFOR, after_help = H_LOOKFOR)]
    LookforComment {
        /// the post file; the suffix may be omitted
        path: PathBuf,
        /// raw case-insensitive substring of body or comment header
        #[arg(long, value_name = "text")]
        keyword: Option<String>,
        /// the comment author
        #[arg(long, value_name = "nick")]
        by: Option<String>,
        /// body mentions @nick
        #[arg(long, value_name = "nick")]
        mention: Option<String>,
        /// body contains @#n
        #[arg(long = "reply-to", value_name = "n")]
        reply_to: Option<u64>,
        /// comment number is n
        #[arg(long, value_name = "n")]
        comment: Option<u64>,
    },
    /// Render a brief's description, notes and target text
    #[command(override_usage = U_READ_BRIEF, after_help = H_READ_BRIEF)]
    ReadBrief {
        /// the brief file; the suffix may be omitted
        path: PathBuf,
        /// render only this entry (title matched after whitespace folding)
        #[arg(long, value_name = "title")]
        entry: Option<String>,
    },
    /// Report the states of DIR, its ancestors and subtree
    #[command(override_usage = U_CHECK_STATE, after_help = H_CHECK_STATE)]
    CheckState {
        /// directory to scan (default: the current directory; recursive, plus ancestors)
        dir: Option<PathBuf>,
        #[command(flatten)]
        walk: WalkArgs,
    },
    /// Wait for any change to one collaboration file
    #[command(override_usage = U_WAIT, after_help = H_WAIT)]
    Wait {
        /// the file to watch; the suffix may be omitted
        path: PathBuf,
    },
    /// Wait for a comment that mentions or replies to you
    #[command(override_usage = U_WAIT_POST, after_help = H_WAIT_POST)]
    WaitPost {
        /// the post file to watch; the suffix may be omitted
        path: PathBuf,
        /// the nick whose mentions/replies end the wait; any change hits when omitted
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
    },
    /// Wait for changes to DIR/state.agent-besties.md
    #[command(override_usage = U_WAIT_STATE, after_help = H_WAIT_STATE)]
    WaitState {
        /// the directory (default: the current directory); only its state file is watched
        dir: Option<PathBuf>,
        /// restrict hits to this status's declarations ('locked', 'observing' or a custom name)
        #[arg(long, value_name = "name")]
        status: Option<String>,
    },
}

impl Commands {
    /// Whether this command walks a tree or ancestor state files (spec §8.4
    /// progress is wired only for these).
    pub fn traverses(&self) -> bool {
        matches!(
            self,
            Commands::List { .. }
                | Commands::ReadNotification { .. }
                | Commands::CheckState { .. }
                | Commands::State { .. }
                | Commands::Profile { .. }
                | Commands::Brief { .. }
        )
    }
}

/// Validate the parsed business command and map it onto a core [`Request`] or
/// a wait invocation. `ctx` supplies the one clock `--since` / `--for` are
/// resolved against. Fails with the exact stderr message (without the
/// `error: ` prefix) for every usage-class condition (spec §7.1.7).
/// `guide` never reaches this function.
pub fn resolve(command: Commands, ctx: &Ctx) -> Result<Invocation, String> {
    match command {
        Commands::List { dir, types, walk } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let walk = walk_options(walk)?;
            let types = types.into_iter().map(kind_of).collect();
            Ok(Invocation::Base {
                command: "list",
                kind: DiagKind::Dir(dir.clone()),
                request: Request::List(ListParams { dir, types, walk }),
            })
        }
        Commands::Check { path } => {
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "check",
                kind: DiagKind::File(path.clone()),
                request: Request::Check(CheckParams { path }),
            })
        }
        Commands::Profile {
            path,
            as_nick,
            section,
            message,
            remove,
        } => {
            let path = suffixed(path);
            let as_nick = opt_nick(as_nick)?;
            let section = opt_section(section)?;
            let action = match (&section, message, remove) {
                (None, None, false) => ProfileAction::None,
                (None, Some(_), _) | (None, _, true) => {
                    return Err("-m and --remove require --section <name>".to_string());
                }
                (Some(_), None, false) => {
                    return Err(
                        "--section requires exactly one of -m <content> or --remove".to_string()
                    );
                }
                (Some(_), Some(_), true) => {
                    return Err(
                        "--section takes exactly one of -m <content> or --remove, not both"
                            .to_string(),
                    );
                }
                (Some(_), Some(m), false) => ProfileAction::SetBody(normalize_message(m)?),
                (Some(_), None, true) => ProfileAction::Remove,
            };
            Ok(Invocation::Base {
                command: "profile",
                kind: DiagKind::File(path.clone()),
                request: Request::Profile(ProfileParams {
                    path,
                    as_nick,
                    section,
                    action,
                }),
            })
        }
        Commands::Post {
            path,
            as_nick,
            message,
            edit,
        } => {
            let nick = nick_value(as_nick, "--as")?;
            let body = normalize_message(message)?;
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "post",
                kind: DiagKind::File(path.clone()),
                request: Request::Post(PostParams {
                    path,
                    nick,
                    body,
                    edit,
                }),
            })
        }
        Commands::Subscribe {
            path,
            as_nick,
            cancel,
        } => {
            let nick = nick_value(as_nick, "--as")?;
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "subscribe",
                kind: DiagKind::File(path.clone()),
                request: Request::Subscribe(SubscribeParams { path, nick, cancel }),
            })
        }
        Commands::Brief {
            path,
            as_nick,
            message,
            target,
            entry,
            regex,
            remove,
            refresh,
        } => {
            let as_nick = opt_nick(as_nick)?;
            let entry = opt_entry(entry)?;
            if let Some(pattern) = &regex {
                if pattern.contains('\n') || pattern.contains('\r') {
                    return Err("--regex must not contain newlines".to_string());
                }
                if let Err(err) = regex::Regex::new(pattern) {
                    return Err(format!(
                        "--regex does not compile: {}",
                        names::fold(&err.to_string())
                    ));
                }
            }
            if let Some(t) = &target {
                if t.contains('\n') || t.contains('\r') {
                    return Err("--target must not contain newlines".to_string());
                }
            }
            if regex.is_some() && target.is_none() {
                return Err("--regex requires --target <path>".to_string());
            }
            let form = if refresh {
                if target.is_some() || message.is_some() || remove {
                    return Err(
                        "--refresh cannot be combined with --target, -m or --remove".to_string()
                    );
                }
                BriefForm::Refresh { entry }
            } else if remove {
                let entry = entry.ok_or_else(|| "--remove requires --entry <title>".to_string())?;
                if message.is_some() {
                    return Err("-m cannot be combined with --remove".to_string());
                }
                if target.is_some() {
                    return Err("--target cannot be combined with --remove".to_string());
                }
                BriefForm::RemoveEntry { entry }
            } else if let Some(t) = target {
                BriefForm::Target(BriefTarget {
                    target: hash::normalized_target(&t),
                    entry,
                    regex: match regex {
                        None => RegexOpt::Keep,
                        Some(p) if p.is_empty() => RegexOpt::Delete,
                        Some(p) => RegexOpt::Set(p),
                    },
                    note: message.map(normalize_message).transpose()?,
                })
            } else if let Some(title) = entry {
                let note = message.ok_or_else(|| {
                    "--entry requires -m (note), --remove or --refresh".to_string()
                })?;
                BriefForm::NoteOnly {
                    entry: title,
                    note: normalize_message(note)?,
                }
            } else if let Some(m) = message {
                BriefForm::SetDescription(normalize_message(m)?)
            } else {
                BriefForm::Bare
            };
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "brief",
                kind: DiagKind::File(path.clone()),
                request: Request::Brief(BriefParams {
                    path,
                    as_nick,
                    form,
                }),
            })
        }
        Commands::State {
            dir,
            as_nick,
            status,
            for_duration,
            remove,
        } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            if let Some(removed) = remove {
                let nick = nick_value(removed, "--remove")?;
                let as_nick = opt_nick(as_nick)?;
                let status = opt_status(status)?;
                if for_duration.is_some() {
                    return Err(
                        "--for is only valid in the declare form (state --as <nick> --status <name>)"
                            .to_string(),
                    );
                }
                Ok(Invocation::Base {
                    command: "state",
                    kind: DiagKind::Dir(dir.clone()),
                    request: Request::State(StateParams::Remove(StateRemove {
                        dir,
                        nick,
                        as_nick,
                        status,
                    })),
                })
            } else {
                let nick = match as_nick {
                    Some(n) => nick_value(n, "--as")?,
                    None => {
                        return Err(
                            "state requires --as <nick> --status <name>, or the remove form --remove <nick>"
                                .to_string(),
                        );
                    }
                };
                let status = match status {
                    Some(s) => status_value(s)?,
                    None => {
                        return Err("--status <name> is required in the declare form".to_string());
                    }
                };
                let for_secs = match for_duration {
                    Some(raw) => Some(duration_secs(&raw, ctx)?),
                    None => None,
                };
                Ok(Invocation::Base {
                    command: "state",
                    kind: DiagKind::Dir(dir.clone()),
                    request: Request::State(StateParams::Declare(StateDeclare {
                        dir,
                        nick,
                        status,
                        for_secs,
                    })),
                })
            }
        }
        Commands::ReadNotification {
            dir,
            as_nick,
            since,
            walk,
        } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let nick = nick_value(as_nick, "--as")?;
            let since = time::since(&since, ctx.now).map_err(usage_message)?;
            let walk = walk_options(walk)?;
            Ok(Invocation::Base {
                command: "read-notification",
                kind: DiagKind::Dir(dir.clone()),
                request: Request::ReadNotification(ReadNotificationParams {
                    dir,
                    nick,
                    since,
                    walk,
                }),
            })
        }
        Commands::LookforComment {
            path,
            keyword,
            by,
            mention,
            reply_to,
            comment,
        } => {
            let keyword = match keyword {
                Some(k) if k.is_empty() => {
                    return Err("--keyword must not be empty".to_string());
                }
                k => k,
            };
            let by = by.map(|n| nick_value(n, "--by")).transpose()?;
            let mention = mention.map(|n| nick_value(n, "--mention")).transpose()?;
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "lookfor-comment",
                kind: DiagKind::File(path.clone()),
                request: Request::LookforComment(LookforCommentParams {
                    path,
                    filters: Filters {
                        keyword,
                        by,
                        mention,
                        reply_to,
                        comment,
                    },
                }),
            })
        }
        Commands::ReadBrief { path, entry } => {
            let entry = opt_entry(entry)?;
            let path = suffixed(path);
            Ok(Invocation::Base {
                command: "read-brief",
                kind: DiagKind::File(path.clone()),
                request: Request::ReadBrief(ReadBriefParams { path, entry }),
            })
        }
        Commands::CheckState { dir, walk } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let walk = walk_options(walk)?;
            Ok(Invocation::Base {
                command: "check-state",
                kind: DiagKind::Dir(dir.clone()),
                request: Request::CheckState(CheckStateParams { dir, walk }),
            })
        }
        Commands::Wait { path } => {
            let path = suffixed(path);
            Ok(Invocation::Wait {
                kind: DiagKind::File(path.clone()),
                path,
                filter: wait::Filter::Any,
            })
        }
        Commands::WaitPost { path, as_nick } => {
            let as_nick = opt_nick(as_nick)?;
            let path = suffixed(path);
            Ok(Invocation::Wait {
                kind: DiagKind::File(path.clone()),
                path,
                filter: wait::Filter::Post(as_nick),
            })
        }
        Commands::WaitState { dir, status } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let status = opt_status(status)?;
            let path = dir.join(format!("state{}", consts::SUFFIX));
            Ok(Invocation::Wait {
                kind: DiagKind::Dir(dir),
                path,
                filter: wait::Filter::State(status),
            })
        }
    }
}

/// Append `consts::SUFFIX` when the user omitted it (spec §2.4.3); a PATH
/// given with the suffix is used as-is. This is exactly
/// [`fsx::walk::managed_path`]: the presence test is an exact byte compare
/// on the file name only (a trailing separator can never cause a second
/// append) and every `OsStr` byte is preserved — no lossy round-trip.
fn suffixed(path: PathBuf) -> PathBuf {
    fsx::walk::managed_path(&path)
}

/// Build and validate the walk options (globs validated before any file
/// access via the Storage entry point).
fn walk_options(walk: WalkArgs) -> Result<fsx::walk::Options, String> {
    let options = fsx::walk::Options {
        include: walk.include,
        exclude: walk.exclude,
        no_ignore: walk.no_ignore,
    };
    fsx::walk::validate(&options).map_err(usage_message)?;
    Ok(options)
}

/// Resolve `--for <duration>` to seconds, rejecting unparseable values and
/// durations that move `now` out of the representable range (spec §1
/// DURATION, §7.1.7).
fn duration_secs(raw: &str, ctx: &Ctx) -> Result<i64, String> {
    let secs = time::duration(raw).map_err(usage_message)?;
    time::until(ctx.now, secs).map_err(usage_message)?;
    Ok(secs)
}

/// Reject Unicode Cc control characters and double spaces (spec §7.1.6).
fn plain_value(raw: &str, flag: &str) -> Result<(), String> {
    if raw.chars().any(|c| c.is_control()) {
        return Err(format!("{flag} must not contain control characters"));
    }
    if raw.contains("  ") {
        return Err(format!("{flag} must not contain two consecutive spaces"));
    }
    Ok(())
}

fn nick_value(raw: String, flag: &str) -> Result<String, String> {
    plain_value(&raw, flag)?;
    if names::valid_nick(&raw) {
        Ok(raw)
    } else {
        Err(format!(
            "{flag} must be a nick: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '{}')",
            names::fold(&raw)
        ))
    }
}

fn opt_nick(value: Option<String>) -> Result<Option<String>, String> {
    value.map(|n| nick_value(n, "--as")).transpose()
}

fn status_value(raw: String) -> Result<String, String> {
    plain_value(&raw, "--status")?;
    if names::valid_status(&raw) {
        Ok(raw)
    } else {
        Err(format!(
            "--status must be a status name: 1-32 characters, a lowercase letter then lowercase letters, digits or '-' (got '{}')",
            names::fold(&raw)
        ))
    }
}

fn opt_status(value: Option<String>) -> Result<Option<String>, String> {
    value.map(status_value).transpose()
}

fn section_value(raw: String) -> Result<String, String> {
    plain_value(&raw, "--section")?;
    if names::valid_section(&raw) {
        Ok(raw)
    } else {
        Err(format!(
            "--section must be a section name: 1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-' (got '{}')",
            names::fold(&raw)
        ))
    }
}

fn opt_section(value: Option<String>) -> Result<Option<String>, String> {
    value.map(section_value).transpose()
}

/// `--entry <title>`: plain-value check on the raw text, then §4.3.1
/// whitespace folding for matching (spec §7.1.6).
fn entry_value(raw: String) -> Result<String, String> {
    plain_value(&raw, "--entry")?;
    Ok(names::fold(&raw))
}

fn opt_entry(value: Option<String>) -> Result<Option<String>, String> {
    value.map(entry_value).transpose()
}

/// `-m` normalization (spec §2.1.3): CRLF folded to LF, trailing
/// whitespace/newlines stripped, must be non-empty afterwards.
fn normalize_message(raw: String) -> Result<String, String> {
    names::normalize_body(&raw).map_err(usage_message)
}

/// The one-line stderr message for a fault from a value validator or the
/// `Ctx` environment entry points: the core-owned `Usage` text, moved out
/// unchanged (spec §7.1.7). Those producers return only `Fault::Usage`; any
/// other variant is an internal inconsistency and is named as such rather
/// than dressed up as a grammar hint.
pub fn usage_message(fault: Fault) -> String {
    match fault {
        Fault::Usage(msg) => msg,
        other => format!("internal error: expected a usage fault, got {other:?}"),
    }
}

const TOP_AFTER_HELP: &str = "\
PATH arguments name a collaboration file and may omit the '.agent-besties.md' suffix; it is appended when missing.
DIR arguments default to the current directory; every scan is recursive.
TIME example: 2026-09-17T09:12:00Z (UTC, seconds precision). DURATION example: 1h30m (Nd/Nh/Nm/Ns in descending order d > h > m > s, each unit at most once, total > 0).
Type vocabulary: post, brief, agent-profile, state ('profile' is accepted as an alias for agent-profile).
'check' reports one file's format, diagnostics and status without target content; 'read-brief' renders a brief including target text; 'check-state' reports the state entries of a directory, its ancestors and its subtree.
For usage judgment see 'agent-coexist guide'; for exact syntax see this help and each command's --help.
Exit codes for business commands: 0 = first line 'ok' or 'warn' ('warn' is also exit 0; parse the first line to detect conflicts); 1 = first line 'error'; 2 = usage error (one stderr line); 3 = retryable lock failure (E-LOCKED/E-INUSE/E-FLOCKED); 130 = only wait, wait-post or wait-state cancelled with Ctrl-C (no stdout).
guide, help [COMMAND], --help and --version are information output: they print Markdown or clap help/version to stdout with no ok/warn/error status line and exit 0 on success; usage errors still exit 2; a failed guide write or flush exits 1.
Progress ('scanning N dirs, M files...') is only written when stderr is a TTY; no output means the scan is still running within its bounded waits.
Help texts and fix commands assume a POSIX shell (Git Bash on Windows); cmd.exe/PowerShell users must adjust quoting.";

// Per-command synopses, shown by clap as each page's only `Usage:` block and
// in grammar-error hints. `   or: ` lines align under the text after `Usage: `.
const U_GUIDE: &str = "agent-coexist guide [start|read|write|wait]";
const U_LIST: &str =
    "agent-coexist list [DIR] [--type <type>]... [--include <glob>]... [--exclude <glob>]... [--no-ignore]";
const U_CHECK: &str = "agent-coexist check <PATH>";
const U_PROFILE: &str =
    "agent-coexist profile <PATH> [--as <nick>] [--section <name> (-m <content> | --remove)]";
const U_POST: &str = "agent-coexist post <PATH> --as <nick> -m <content> [--edit <n>]";
const U_SUBSCRIBE: &str = "agent-coexist subscribe <PATH> --as <nick> [--cancel]";
const U_BRIEF: &str = "\
agent-coexist brief <PATH> [--as <nick>]
   or: agent-coexist brief <PATH> [--as <nick>] -m <description>
   or: agent-coexist brief <PATH> [--as <nick>] --target <path> [--entry <title>] [--regex <pattern>] [-m <note>]
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> -m <note>
   or: agent-coexist brief <PATH> [--as <nick>] --entry <title> --remove
   or: agent-coexist brief <PATH> [--as <nick>] [--entry <title>] --refresh";
const U_STATE: &str = "\
agent-coexist state [DIR] --as <nick> --status <name> [--for <duration>]
   or: agent-coexist state [DIR] --remove <nick> [--as <nick>] [--status <name>]";
const U_READ_NOTIFICATION: &str =
    "agent-coexist read-notification [DIR] --as <nick> --since <time|duration> \
[--include <glob>]... [--exclude <glob>]... [--no-ignore]";
const U_LOOKFOR: &str = "agent-coexist lookfor-comment <PATH> [--keyword <text>] [--by <nick>] \
[--mention <nick>] [--reply-to <n>] [--comment <n>]";
const U_READ_BRIEF: &str = "agent-coexist read-brief <PATH> [--entry <title>]";
const U_CHECK_STATE: &str =
    "agent-coexist check-state [DIR] [--include <glob>]... [--exclude <glob>]... [--no-ignore]";
const U_WAIT: &str = "agent-coexist wait <PATH>";
const U_WAIT_POST: &str = "agent-coexist wait-post <PATH> [--as <nick>]";
const U_WAIT_STATE: &str = "agent-coexist wait-state [DIR] [--status <name>]";

const H_GUIDE: &str = "\
Prints one offline Markdown page to stdout and exits 0. Omit the topic for the index page; pass exactly one of start, read, write, wait for that page. stdout is the page; stderr is empty. No ok/warn/error status line, pager, color, network, or workspace access.
Topics are exact lowercase words; there is no 'index', 'overview' or 'all' topic and no prefix match. This command has no DIR, PATH, --as, or -m. Invalid AGENT_COEXIST_NOW / AGENT_COEXIST_LOCK_WAIT_MS do not block it.
Example: agent-coexist guide start";

const H_LIST: &str = "\
Lists every collaboration file under DIR (default: the current directory) as a tree with a type column. This command has no -m.
--type filters by file type; repeats form a union (post, brief, agent-profile, state; 'profile' is an alias for agent-profile); no --type shows all four. 'busy' and '?' leaves stay shown and counted under any --type, because their type is unknown.
--include/--exclude are gitignore-style globs relative to DIR; --include cannot reopen directories pruned by ignore rules — pair it with --no-ignore.
Type column legend: 'busy' = a file lock or stable read was unavailable, or a Windows open handle denied sharing (close that denying handle and retry); '?' = unreadable (permissions, not UTF-8, invalid H1, or over the managed-file size limit).
Reading uses shared locks with a 100ms wait; busy files become 'busy' leaves and the first line becomes 'warn', never a failure.
Example: agent-coexist list docs --type post --type brief";

const H_CHECK: &str = "\
Reports one file: 'file', 'type' (when determinable), diagnostics in line order, then counts and previews (topic/latest for posts, entries for briefs, sections for profiles, entry rows for state).
Does not expand target-file contents; use read-brief when you need the target text.
PATH is a single collaboration file (the '.agent-besties.md' suffix may be omitted); 'check-state' is the directory-level counterpart for state files. This command has no -m.
Reading uses a shared lock with a 1s wait; a busy file fails with E-LOCKED (exit 3, retry the same command shortly).
Example: agent-coexist check docs/standup";

const H_PROFILE: &str = "\
-m means the body of --section for this command; it is stored verbatim in a code fence. --remove deletes the whole --section. Without --section the file is created (H1 only) or reported 'unchanged'.
--as is your nick, used only for the functional-lock check and never written. With --as, another nick's valid 'locked' on the file's directory or an ancestor triggers E-FLOCKED; without --as, every such 'locked' does, including one you declared yourself.
Writing takes an exclusive lock with a 200ms fast-fail (E-LOCKED exit 3). On E-FLOCKED, retry at the expiry time shown in 'reason'; do not poll at E-LOCKED's millisecond cadence.
Example: agent-coexist profile agents/alice --section role -m 'Rust implementer. Owns crates/core.'";

const H_POST: &str = "\
-m means the comment body for this command; it is stored verbatim in a code fence ('comment #n' names what was written). --as is the author nick recorded in the comment header. --edit rewrites your own comment #n (its author must match --as).
Writing takes an exclusive lock and queues up to 5s (override with AGENT_COEXIST_LOCK_WAIT_MS); a still-busy file fails with E-LOCKED (exit 3, retry the same command shortly).
Example: agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs'";

const H_SUBSCRIBE: &str = "\
Adds --as to the post's subscriber list; --cancel removes only --as. Subscribing twice or cancelling a non-member leaves the file unchanged ('unchanged <nick>'). This command has no -m.
Locking behaves like post: exclusive, queues up to 5s (AGENT_COEXIST_LOCK_WAIT_MS overrides), then E-LOCKED exit 3.
Example: agent-coexist subscribe docs/standup --as alice";

const H_BRIEF: &str = "\
The six usage forms are mutually exclusive, in order: create the skeleton, else 'unchanged'; create with or replace the description; add or update an entry; replace only the entry's note; delete the entry; recompute hashes (every entry, or only --entry).
-m means the description when no --target/--entry is given, and the entry's note otherwise; 'description set' / 'note set' name what was written.
--regex requires --target. Writing checks only that the pattern compiles; a target it does not match makes the entry 'stale' (regex not found) in check and read-brief. --regex '' deletes every '- regex:' line of the entry, including hand-written duplicates. --regex and --target must not contain raw CR or LF (use \\r / \\n in a pattern to match those bytes). --remove requires --entry and excludes --target, -m. --refresh excludes --target, -m and --remove.
--as is your nick, used only for the functional-lock check and never written. With --as, another nick's valid 'locked' on the file's directory or an ancestor triggers E-FLOCKED; without --as, every such 'locked' does, including one you declared yourself.
Writing takes an exclusive lock with a 200ms fast-fail (E-LOCKED exit 3). On E-FLOCKED, retry at the expiry time shown in 'reason'; do not poll at E-LOCKED's millisecond cadence.
Example: agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'";

const H_STATE: &str = "\
The first usage form declares a status. The second removes a nick's entries: with --status only under that status, without it under every status, including a live 'locked' declaration.
DIR defaults to the current directory and must already exist (it is never created). This command has no -m.
Built-in statuses: 'locked' = please do not change this directory or its subdirectories until the end time; 'observing' = I am watching, changes are welcome. Custom names are stored and shown but carry no lock meaning.
--for sets the duration (e.g. 1h30m); default 2h for 'locked', 8h for anything else. In the remove form --as defaults to the removed nick (self-removal); a different --as adds a 'removed-other' warn row.
Writing takes an exclusive lock with a 200ms fast-fail; others' 'locked' never blocks a state write — conflicts are reported as rows and only turn the first line 'warn' (exit stays 0).
Example: agent-coexist state src --as alice --status locked --for 1h30m";

const H_READ_NOTIFICATION: &str = "\
DIR defaults to the current directory; the scan is recursive and also reads ancestor state files. This command has no -m and never writes files.
--since is an RFC 3339 time (e.g. 2026-09-17T09:12:00Z) or a duration back from now (e.g. 1h30m).
--include/--exclude are gitignore-style globs relative to DIR; --include cannot reopen directories pruned by ignore rules — pair it with --no-ignore.
Scanning uses shared 100ms waits; busy or unparseable files become 'skipped'/'partial' rows and the first line becomes 'warn'.
Example: agent-coexist read-notification --as alice --since 1h";

const H_LOOKFOR: &str = "\
No filter lists every comment; several filters intersect. This command has no -m.
--keyword matches the raw body/header text as a case-insensitive substring (bodies may contain consecutive spaces; the value is used as given, never folded).
--by is the comment's author (the 'authors' key of check output lists a file's authors). --mention matches a body mention @nick; --reply-to matches a body @#n; --comment selects seq n.
Reading uses a shared lock with a 1s wait; a busy file fails with E-LOCKED (exit 3).
Example: agent-coexist lookfor-comment docs/standup --by bob";

const H_READ_BRIEF: &str = "\
Renders the description and, per entry, its state, note and the target's content as verbatim text blocks. --entry renders only that entry (titles match after whitespace folding). Use check when you only need status, not the target text. This command has no -m.
Reading uses a shared lock with a 1s wait; a busy file fails with E-LOCKED (exit 3).
Example: agent-coexist read-brief docs/onboarding --entry 'parser entry point'";

const H_CHECK_STATE: &str = "\
Reports the state entries of DIR itself, its ancestors ('inherited' rows) and its subtree; 'check' is the single-file counterpart. This command has no -m.
--include/--exclude are gitignore-style globs relative to DIR; --include cannot reopen directories pruned by ignore rules — pair it with --no-ignore.
Each entry row can end with a mark: 'conflict' for a valid 'locked' entry when another nick holds a valid 'locked' on an overlapping directory (equal, ancestor or descendant; both entries are marked, inherited ones included), or 'expired' for an own or subtree entry that is no longer valid ('inherited' rows list valid entries only). Either mark makes the first line 'warn' (exit 0).
Scanning uses shared 100ms waits; busy or unparseable state files become 'skipped'/'partial' rows and the first line becomes 'warn'.
Example: agent-coexist check-state src";

const H_WAIT: &str = "\
Waits until the file changes (any of the four types), then prints one diff report and exits 0. The file is sampled once per second; nothing is ever written.
Once the baseline is established, a single 'waiting  wait  <object>' line is written to stderr. Ctrl-C cancels: exit 130, no stdout.
Each sample takes a shared lock with a 1s wait and releases it before the next; no lock is held between samples. A lock still busy after that wait ends the wait with E-LOCKED (exit 3) instead of being retried; a file another program holds open without sharing (Windows) ends it with E-INUSE (exit 3). Other failures use the exit codes listed in 'agent-coexist --help'. PATH may omit the '.agent-besties.md' suffix.
Example: agent-coexist wait docs/onboarding";

const H_WAIT_POST: &str = "\
Waits until a comment in this post directly mentions @<nick> or replies to a comment authored by <nick>; without --as any change hits.
A single 'waiting  wait-post  <object>' line is written to stderr once the baseline is established. Ctrl-C cancels: exit 130, no stdout.
The post is sampled once per second; each sample takes a shared lock with a 1s wait and releases it before the next. A lock still busy after that wait ends the wait with E-LOCKED (exit 3) instead of being retried; a file another program holds open without sharing (Windows) ends it with E-INUSE (exit 3).
Example: agent-coexist wait-post docs/standup --as alice";

const H_WAIT_STATE: &str = "\
Watches exactly DIR/state.agent-besties.md (DIR defaults to the current directory); ancestors and the subtree are not watched.
--status ('locked', 'observing' or a valid custom name) restricts hits to that status's declarations; without it any change to the file hits.
A single 'waiting  wait-state  <object>' line is written to stderr once the baseline is established. Ctrl-C cancels: exit 130, no stdout.
The state file is sampled once per second; each sample takes a shared lock with a 1s wait and releases it before the next. A lock still busy after that wait ends the wait with E-LOCKED (exit 3) instead of being retried; a file another program holds open without sharing (Windows) ends it with E-INUSE (exit 3).
Example: agent-coexist wait-state src --status locked";

#[cfg(test)]
mod tests {
    use super::*;

    /// §2.4.3 argument-path identity: an already-suffixed PATH is used
    /// byte-for-byte as given — a trailing separator never causes a second
    /// append (regression: CLI-SUFFIXED-WHOLE-PATH).
    #[test]
    fn suffixed_path_with_trailing_separator_is_used_as_is() {
        let path = PathBuf::from("docs/standup.agent-besties.md/");
        let resolved = suffixed(path.clone());
        assert_eq!(resolved.as_os_str(), path.as_os_str());
        assert_eq!(
            resolved.file_name().and_then(|name| name.to_str()),
            Some("standup.agent-besties.md")
        );
    }

    /// The suffix test targets the file name only, so a plain name still
    /// gets exactly one append.
    #[test]
    fn suffixed_appends_once_to_plain_name() {
        assert_eq!(
            suffixed(PathBuf::from("docs/standup")),
            PathBuf::from("docs/standup.agent-besties.md")
        );
    }

    /// Non-UTF-8 components survive resolution byte-for-byte — no lossy
    /// round-trip through U+FFFD (regression: CLI-SUFFIXED-WHOLE-PATH).
    #[cfg(unix)]
    #[test]
    fn suffixed_preserves_non_utf8_bytes() {
        use std::os::unix::ffi::OsStrExt;

        let parent = std::ffi::OsStr::from_bytes(b"docs\xff");
        let path = PathBuf::from(parent).join("standup");
        let mut expected = path.as_os_str().as_bytes().to_vec();
        expected.extend_from_slice(consts::SUFFIX.as_bytes());
        assert_eq!(suffixed(path).as_os_str().as_bytes(), expected.as_slice());

        // Already suffixed: untouched identity, not even a rebuild.
        let path =
            PathBuf::from(parent).join(std::ffi::OsStr::from_bytes(b"standup.agent-besties.md"));
        assert_eq!(suffixed(path.clone()), path);
    }

    fn parse_args(args: &[&str]) -> Result<Cli, clap::Error> {
        let mut argv = Vec::with_capacity(args.len() + 1);
        argv.push("agent-coexist");
        argv.extend(args);
        Cli::try_parse_from(&argv)
    }

    fn brief_command(target: Option<String>, regex: Option<String>) -> Commands {
        Commands::Brief {
            path: PathBuf::from("plan"),
            as_nick: None,
            message: None,
            target,
            entry: None,
            regex,
            remove: false,
            refresh: false,
        }
    }

    fn resolve_brief(target: Option<String>, regex: Option<String>) -> Result<Invocation, String> {
        let ctx = Ctx::new(Some(String::new()), None).expect("ctx");
        resolve(brief_command(target, regex), &ctx)
    }

    fn brief_target(inv: Invocation) -> BriefTarget {
        match inv {
            Invocation::Base {
                request: Request::Brief(p),
                ..
            } => match p.form {
                BriefForm::Target(t) => t,
                _ => panic!("expected brief target form"),
            },
            _ => panic!("expected base brief invocation"),
        }
    }

    /// Raw CR/LF in --regex/--target are usage failures at resolve, before
    /// any file access (R4 raw-CR-regex write produced ok + unparseable).
    #[test]
    fn resolve_rejects_raw_cr_or_lf_in_regex_and_target() {
        assert!(resolve_brief(Some("target.txt".into()), Some("\rmatch".into())).is_err());
        assert!(resolve_brief(Some("target.txt".into()), Some("a\nmatch".into())).is_err());
        assert!(resolve_brief(Some("target.txt".into()), Some("a\r\nmatch".into())).is_err());
        assert!(resolve_brief(Some("tar\rget.txt".into()), Some("ok".into())).is_err());
        assert!(resolve_brief(Some("tar\nget.txt".into()), None).is_err());
        assert!(resolve_brief(Some("tar\r\nget.txt".into()), None).is_err());
    }

    /// Two-character regex escapes and surrounding whitespace stay usable;
    /// values are not trimmed and other controls are not newly rejected.
    #[test]
    fn resolve_keeps_regex_escapes_and_raw_whitespace() {
        let cr =
            brief_target(resolve_brief(Some("target.txt".into()), Some("\\r".into())).unwrap());
        match cr.regex {
            RegexOpt::Set(p) => assert_eq!(p, "\\r"),
            _ => panic!("expected Set"),
        }
        let lf =
            brief_target(resolve_brief(Some("target.txt".into()), Some("\\n".into())).unwrap());
        match lf.regex {
            RegexOpt::Set(p) => assert_eq!(p, "\\n"),
            _ => panic!("expected Set"),
        }
        let padded =
            brief_target(resolve_brief(Some("a.txt ".into()), Some(" foo ".into())).unwrap());
        assert_eq!(padded.target, "a.txt ");
        match padded.regex {
            RegexOpt::Set(p) => assert_eq!(p, " foo "),
            _ => panic!("expected Set"),
        }
        let tabbed =
            brief_target(resolve_brief(Some("a.txt".into()), Some("\tfoo".into())).unwrap());
        match tabbed.regex {
            RegexOpt::Set(p) => assert_eq!(p, "\tfoo"),
            _ => panic!("expected Set"),
        }
        let delete =
            brief_target(resolve_brief(Some("a.txt".into()), Some(String::new())).unwrap());
        assert!(matches!(delete.regex, RegexOpt::Delete));
    }

    /// `--entry --remove --target` is a usage error at resolve, never
    /// RemoveEntry. Valid `--entry --remove` stays representable
    /// (regression: BRIEF-REMOVE-DROPS-TARGET).
    #[test]
    fn resolve_rejects_remove_combined_with_target() {
        let ctx = Ctx::new(Some(String::new()), None).expect("ctx");
        let mixed = Commands::Brief {
            path: PathBuf::from("plan"),
            as_nick: None,
            message: None,
            target: Some("new.txt".into()),
            entry: Some("item".into()),
            regex: None,
            remove: true,
            refresh: false,
        };
        match resolve(mixed, &ctx) {
            Err(_) => {}
            Ok(Invocation::Base {
                request: Request::Brief(p),
                ..
            }) => panic!("expected usage error, got form {:?}", p.form),
            Ok(_) => panic!("expected usage error, never RemoveEntry"),
        }

        let valid = Commands::Brief {
            path: PathBuf::from("plan"),
            as_nick: None,
            message: None,
            target: None,
            entry: Some("item".into()),
            regex: None,
            remove: true,
            refresh: false,
        };
        match resolve(valid, &ctx) {
            Ok(Invocation::Base {
                request: Request::Brief(p),
                ..
            }) => match p.form {
                BriefForm::RemoveEntry { entry } => assert_eq!(entry, "item"),
                other => panic!("expected RemoveEntry, got {other:?}"),
            },
            _ => panic!("expected ok remove"),
        }
    }

    /// Guide grammar: omitted topic is the default page; only exact
    /// lowercase start|read|write|wait; unknown topic/args/flags are clap
    /// usage errors; business commands stay on the business branch.
    #[test]
    fn guide_grammar_is_exact_and_does_not_capture_business() {
        match parse_args(&["guide"]) {
            Ok(Cli {
                command: TopCommand::Guide { topic: None },
            }) => {}
            other => panic!("expected default guide, got {other:?}"),
        }
        for (word, want) in [
            ("start", GuideTopic::Start),
            ("read", GuideTopic::Read),
            ("write", GuideTopic::Write),
            ("wait", GuideTopic::Wait),
        ] {
            match parse_args(&["guide", word]) {
                Ok(Cli {
                    command: TopCommand::Guide { topic: Some(got) },
                }) if got == want => {}
                other => panic!("expected guide {word}, got {other:?}"),
            }
        }

        for bad in ["index", "overview", "all", "Start", "WAIT"] {
            let err = parse_args(&["guide", bad]).expect_err(bad);
            assert!(err.use_stderr(), "{bad}");
        }

        assert!(parse_args(&["guide", "start", "extra"])
            .unwrap_err()
            .use_stderr());
        assert!(parse_args(&["guide", "--as", "alice"])
            .unwrap_err()
            .use_stderr());
        assert!(parse_args(&["guide", "--json"]).unwrap_err().use_stderr());
        assert!(parse_args(&["read"]).unwrap_err().use_stderr());
        assert!(parse_args(&["start"]).unwrap_err().use_stderr());

        match parse_args(&["wait", "docs/onboarding"]) {
            Ok(Cli {
                command: TopCommand::Business(Commands::Wait { .. }),
            }) => {}
            other => panic!("wait must stay a business command, got {other:?}"),
        }
        match parse_args(&["list"]) {
            Ok(Cli {
                command: TopCommand::Business(Commands::List { .. }),
            }) => {}
            other => panic!("list must stay a business command, got {other:?}"),
        }
    }
}
