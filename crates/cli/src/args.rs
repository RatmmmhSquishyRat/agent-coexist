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

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, Args, Command, CommandFactory, Parser, Subcommand};

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
    arg_required_else_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: TopCommand,
}

/// Top-level dispatch: `guide`, or one of the flattened business [`Commands`].
#[derive(Subcommand, Debug)]
pub enum TopCommand {
    /// Print a built-in guide page on when and how to use coexist
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
            "'{}' is not a valid guide topic; expected start, read, write or wait{}",
            names::echo_raw(raw),
            edge_fix(raw, |v| parse_guide_topic(v).is_ok())
        )),
    }
}

/// The `WALK` argument group shared by `list`, `read-notification` and
/// `check-state` (spec §7.2).
#[derive(Args, Debug)]
pub struct WalkArgs {
    /// keep only files matching this gitignore-style glob, relative to DIR (repeatable; does not reach into ignored directories without --no-ignore)
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
            "'{}' is not a valid type; expected {TYPE_RULE}{}",
            names::echo_raw(raw),
            edge_fix(raw, |v| parse_type(v).is_ok())
        )),
    }
}

/// `; remove the space before or after '<word>'` when only edge
/// whitespace keeps `raw` from being accepted, else nothing.
fn edge_fix(raw: &str, valid: impl Fn(&str) -> bool) -> String {
    names::edge_space_fix(raw, valid)
        .map(|fix| format!("; {fix}"))
        .unwrap_or_default()
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
    /// Check one file: format, diagnostics and a preview
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
        /// your nick; not written, only exempts your own 'locked' declarations
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the section to set or remove
        #[arg(long, value_name = "name")]
        section: Option<String>,
        /// the section body, stored verbatim in a code fence
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
        /// the author nick written in the comment heading
        #[arg(long = "as", value_name = "nick")]
        as_nick: String,
        /// the comment body, stored verbatim in a code fence
        #[arg(short = 'm', value_name = "content")]
        message: String,
        /// replace the whole body of your own comment #n (its author must be --as)
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
        /// your nick; not written, only exempts your own 'locked' declarations
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the description (no --target/--entry) or the entry's note (with --target/--entry)
        #[arg(short = 'm', value_name = "content")]
        message: Option<String>,
        /// the file this entry points to, relative to the brief's directory
        #[arg(long, value_name = "path")]
        target: Option<String>,
        /// the entry title; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
        #[arg(long, value_name = "title")]
        entry: Option<String>,
        /// a regex the target must still match when the brief is read ('' deletes it; needs --target)
        #[arg(long, value_name = "pattern")]
        regex: Option<String>,
        /// delete the whole --entry
        #[arg(long)]
        remove: bool,
        /// recompute stored hashes from the targets' current bytes
        #[arg(long)]
        refresh: bool,
    },
    /// Declare or remove directory state (locks, observations)
    #[command(override_usage = U_STATE, after_help = H_STATE)]
    State {
        /// the directory (default: the current directory); must already exist
        dir: Option<PathBuf>,
        /// declare form: your nick; remove form: who removes (default: the removed nick)
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
        /// the status name; required in the declare form; in the remove form, a filter (omitted: every status)
        #[arg(long, value_name = "name")]
        status: Option<String>,
        /// how long the declaration lasts (declare form only; default 2h for locked, 8h otherwise)
        #[arg(long = "for", value_name = "duration")]
        for_duration: Option<String>,
        /// remove this nick's declarations instead of declaring
        #[arg(long, value_name = "nick")]
        remove: Option<String>,
    },
    /// List mentions, replies, subscriptions and state changes since a time
    #[command(override_usage = U_READ_NOTIFICATION, after_help = H_READ_NOTIFICATION)]
    ReadNotification {
        /// directory to scan (default: the current directory; recursive, plus ancestor state files)
        dir: Option<PathBuf>,
        /// your nick: whose mentions, replies and subscriptions to list
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
        /// case-insensitive text in the comment body or heading, matched as typed
        #[arg(long, value_name = "text")]
        keyword: Option<String>,
        /// the comment author
        #[arg(long, value_name = "nick")]
        by: Option<String>,
        /// the body mentions @nick
        #[arg(long, value_name = "nick")]
        mention: Option<String>,
        /// the body replies to comment n (@#n)
        #[arg(long = "reply-to", value_name = "n")]
        reply_to: Option<u64>,
        /// the comment numbered n
        #[arg(long, value_name = "n")]
        comment: Option<u64>,
    },
    /// Print a brief's description, notes and target text
    #[command(override_usage = U_READ_BRIEF, after_help = H_READ_BRIEF)]
    ReadBrief {
        /// the brief file; the suffix may be omitted
        path: PathBuf,
        /// print only this entry; one space at either end is ignored, while two spaces in a row anywhere (including at an end) or a control character such as a tab or CR is rejected (titles in the file match with whitespace runs read as one space)
        #[arg(long, value_name = "title")]
        entry: Option<String>,
    },
    /// Show the state declarations for DIR, its ancestors and its subtree
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
        /// the nick whose mentions and replies end the wait; without it, any change counts
        #[arg(long = "as", value_name = "nick")]
        as_nick: Option<String>,
    },
    /// Wait for changes to DIR/state.agent-besties.md
    #[command(override_usage = U_WAIT_STATE, after_help = H_WAIT_STATE)]
    WaitState {
        /// the directory (default: the current directory); only its state file is watched
        dir: Option<PathBuf>,
        /// count only changes to declarations under this status ('locked', 'observing' or another name)
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
            // Combination checks come before the --regex/--target value
            // checks, so a value is only validated in a form that uses it
            // and the first message never names an option the next run
            // rejects.
            let form = if refresh {
                if target.is_some() || regex.is_some() || message.is_some() || remove {
                    return Err(
                        "--refresh cannot be combined with --target, --regex, -m or --remove"
                            .to_string(),
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
                if regex.is_some() {
                    return Err("--regex cannot be combined with --remove".to_string());
                }
                BriefForm::RemoveEntry { entry }
            } else if regex.is_some() && target.is_none() {
                return Err("--regex requires --target <path>".to_string());
            } else if let Some(t) = target {
                if let Some(pattern) = &regex {
                    if pattern.contains('\n') || pattern.contains('\r') {
                        // The rejected pattern is echoed so the invisible
                        // CR/LF is visible; trimming is no repair here (it
                        // would change what the pattern matches), so the
                        // advice stays writing the escape instead.
                        return Err(format!(
                            "--regex must not contain raw CR or LF (got '{}'); write \\r or \\n in the pattern to match them",
                            names::echo_raw(pattern)
                        ));
                    }
                    if let Err(err) = regex::Regex::new(pattern) {
                        return Err(format!(
                            "--regex '{}' does not compile: {}",
                            names::echo_raw(pattern),
                            names::regex_reason(&err)
                        ));
                    }
                }
                if t.contains('\n') || t.contains('\r') {
                    // Like the plain-value checks: the rejected value is
                    // echoed with the CR/LF visible, and when trimming
                    // alone repairs it (a CRLF script's trailing CR),
                    // that repair follows and the retry resolves.
                    return Err(format!(
                        "--target must not contain raw CR or LF {}",
                        names::got_raw(&t, |v| !v.contains(['\r', '\n']))
                    ));
                }
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
                    "--entry requires -m (note), --target <path>, --remove or --refresh".to_string()
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
                    return Err(
                        "--keyword must not be empty; omit --keyword to list every comment"
                            .to_string(),
                    );
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

/// What a plain value must not contain (spec §7.1.6), if it does: Unicode
/// Cc control characters, else two consecutive spaces.
fn plain_fault(raw: &str) -> Option<&'static str> {
    if raw.chars().any(char::is_control) {
        Some("control characters")
    } else if raw.contains("  ") {
        Some("two consecutive spaces")
    } else {
        None
    }
}

/// Plain-value check for an entry title: a title has no grammar of its
/// own, so any single-spaced, control-free text is valid and the plain
/// fault is the whole rule. The rejected value is echoed as typed and,
/// when trimming its ends alone yields a value `valid` accepts, the
/// edge-whitespace repair follows, so a CR left by a CRLF script or a
/// doubled space at an end is visible here exactly as on the grammar
/// path.
fn plain_value(raw: &str, flag: &str, valid: impl Fn(&str) -> bool) -> Result<(), String> {
    match plain_fault(raw) {
        Some(fault) => Err(format!(
            "{flag} must not contain {fault} {}",
            names::got_raw(raw, valid)
        )),
        None => Ok(()),
    }
}

/// The plain-value message for a value whose grammar admits no space or
/// control character at all (a nick, status or section name): used only
/// when trimming the ends alone repairs the value, so the one repair
/// the message names is complete. Otherwise `None` and the caller's
/// grammar message states the actual rule instead — `must not contain
/// two consecutive spaces` names a narrower fault whose obvious fix, a
/// single space, the next run still rejects.
fn plain_fault_message(
    raw: &str,
    flag: &str,
    valid: impl Fn(&str) -> bool + Copy,
) -> Option<String> {
    plain_fault(raw)?;
    names::edge_space_fix(raw, valid)?;
    plain_value(raw, flag, valid).err()
}

/// NICK and SECTION grammar in words (spec §2.3), shared by the value
/// validators and the leading-`-` tip so both state the same rule.
const NICK_RULE: &str =
    "1-64 characters, the first a letter, digit or '_', the rest letters, digits, '_' or '-'";
/// STATUS grammar in words (spec §2.3).
const STATUS_RULE: &str =
    "1-32 characters, a lowercase letter then lowercase letters, digits or '-'";
/// The `--type` words (spec §7.1.8).
const TYPE_RULE: &str = "post, brief, agent-profile (alias: profile) or state";

fn nick_value(raw: String, flag: &str) -> Result<String, String> {
    if let Some(message) = plain_fault_message(&raw, flag, names::valid_nick) {
        return Err(message);
    }
    if names::valid_nick(&raw) {
        Ok(raw)
    } else {
        // `--mention @alice` is the natural slip: the body syntax leaks
        // into the option value.
        let hint = match raw.strip_prefix('@') {
            Some(bare) if names::valid_nick(bare) => format!("; write '{bare}' without the '@'"),
            _ => String::new(),
        };
        Err(format!(
            "{flag} must be a nick: {NICK_RULE} {}{hint}",
            names::got_raw(&raw, names::valid_nick)
        ))
    }
}

fn opt_nick(value: Option<String>) -> Result<Option<String>, String> {
    value.map(|n| nick_value(n, "--as")).transpose()
}

fn status_value(raw: String) -> Result<String, String> {
    if let Some(message) = plain_fault_message(&raw, "--status", names::valid_status) {
        return Err(message);
    }
    if names::valid_status(&raw) {
        Ok(raw)
    } else {
        Err(format!(
            "--status must be a status name: {STATUS_RULE} {}",
            names::got_raw(&raw, names::valid_status)
        ))
    }
}

fn opt_status(value: Option<String>) -> Result<Option<String>, String> {
    value.map(status_value).transpose()
}

fn section_value(raw: String) -> Result<String, String> {
    if let Some(message) = plain_fault_message(&raw, "--section", names::valid_section) {
        return Err(message);
    }
    if names::valid_section(&raw) {
        Ok(raw)
    } else {
        Err(format!(
            "--section must be a section name: {NICK_RULE} {}",
            names::got_raw(&raw, names::valid_section)
        ))
    }
}

fn opt_section(value: Option<String>) -> Result<Option<String>, String> {
    value.map(section_value).transpose()
}

/// `--entry <title>`: plain-value check on the raw text, then §4.3.1
/// whitespace folding for matching (spec §7.1.6). A title has no grammar
/// of its own, so the trimmed value is a valid retry exactly when it
/// passes the same plain-value check.
fn entry_value(raw: String) -> Result<String, String> {
    plain_value(&raw, "--entry", |v| plain_fault(v).is_none())?;
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

/// The one-line text for a clap grammar error (without `error: `), with
/// the advice clap cannot phrase for this grammar corrected from the
/// error's structured context and the native command line `words`
/// (program name first). For an unknown option on a business or guide
/// command, clap's `-- <word>` tip turns the word into the PATH/DIR, which
/// is almost never what was meant, so [`unknown_option_tip`] replaces it.
/// A value that is not valid Unicode gets a message that names the
/// argument holding it and how to repair it.
pub fn grammar_error_message(mut error: clap::Error, words: &[OsString]) -> String {
    match error.kind() {
        ErrorKind::InvalidUtf8 => return invalid_unicode_message(words),
        ErrorKind::UnknownArgument => {
            let reported = match error.get(ContextKind::InvalidArg) {
                Some(ContextValue::String(reported)) => Some(reported.clone()),
                _ => None,
            };
            let has_similar = error.get(ContextKind::SuggestedArg).is_some();
            if let Some(tip) =
                reported.and_then(|reported| unknown_option_tip(words, &reported, has_similar))
            {
                error.insert(
                    ContextKind::Suggested,
                    ContextValue::StyledStrs(vec![tip.into()]),
                );
            }
        }
        _ => {}
    }
    // Clap quotes the rejected word as typed; show it the same way as the
    // validators do, so an edge space stays visible and a control
    // character cannot reach the terminal or split the line.
    for kind in [
        ContextKind::InvalidValue,
        ContextKind::InvalidArg,
        ContextKind::InvalidSubcommand,
    ] {
        if let Some(ContextValue::String(word)) = error.get(kind) {
            let shown = names::echo_raw(word);
            if shown != *word {
                error.insert(kind, ContextValue::String(shown));
            }
        }
    }
    let message = error.to_string();
    match message.strip_prefix("error: ") {
        Some(rest) => rest.to_string(),
        None => message,
    }
}

/// Run `f` on the business (or guide) subcommand that `words[1]` names.
fn with_subcommand<R>(words: &[OsString], f: impl FnOnce(&Command) -> Option<R>) -> Option<R> {
    let name = words.get(1)?.to_str()?;
    let mut cli = Cli::command();
    cli.build();
    f(cli.find_subcommand(name)?)
}

/// The option a flag-shaped `word` names on `cmd`, and whether that
/// option still expects its value in the next word (no `=value` or
/// attached short value). `None` for a word naming no option of `cmd`.
fn option_named<'a>(cmd: &'a Command, word: &str) -> Option<(&'a Arg, bool)> {
    if let Some(long) = word.strip_prefix("--") {
        let (name, attached) = match long.split_once('=') {
            Some((name, _)) => (name, true),
            None => (long, false),
        };
        let arg = cmd
            .get_arguments()
            .find(|arg| arg.get_long() == Some(name))?;
        Some((arg, !attached && arg.get_action().takes_values()))
    } else {
        let mut chars = word.strip_prefix('-')?.chars();
        let short = chars.next()?;
        let arg = cmd
            .get_arguments()
            .find(|arg| arg.get_short() == Some(short))?;
        Some((
            arg,
            chars.as_str().is_empty() && arg.get_action().takes_values(),
        ))
    }
}

/// How the option is written in messages: `--long`, else `-s`.
fn option_flag(arg: &Arg) -> String {
    match (arg.get_long(), arg.get_short()) {
        (Some(long), _) => format!("--{long}"),
        (None, Some(short)) => format!("-{short}"),
        (None, None) => arg.get_id().as_str().to_string(),
    }
}

/// Whether `text` is the word clap `reported` as unknown: the long name
/// up to any `=value`, or the short option that starts a single-dash word.
fn is_reported_word(text: &str, reported: &str) -> bool {
    if reported.starts_with("--") {
        text == reported
            || text
                .strip_prefix(reported)
                .is_some_and(|rest| rest.starts_with('='))
    } else {
        !text.starts_with("--") && text.starts_with(reported)
    }
}

/// Replacement tip for the unknown option clap `reported`, found by
/// replaying the grammar over `words` (scanning stops at `--`, after
/// which clap reads no options). `None` keeps clap's own advice: the word
/// is not found, or clap already names a similar option (`has_similar`)
/// and there is nothing more specific to say.
///
/// - Right after an option still expecting its value: [`value_tip`].
/// - `-V` / `--version` after a command: version is top-level only.
/// - A single-dash word naming a long option (`-as`): write `--as`.
/// - Otherwise the command has no such option; `--help` lists them. The
///   `--` form is offered only when the command's PATH/DIR is still
///   free, and only as a conditional for a word that really is that name.
fn unknown_option_tip(words: &[OsString], reported: &str, has_similar: bool) -> Option<String> {
    with_subcommand(words, |cmd| {
        let args = words.get(2..)?;
        let mut pending: Option<&Arg> = None;
        let mut positional_given = false;
        let mut unknown = None;
        for (index, word) in args.iter().enumerate() {
            let text = word.to_string_lossy();
            if text == "--" {
                return None;
            }
            let flag_shaped = text.starts_with('-') && text != "-";
            if let Some(option) = pending.take() {
                if !flag_shaped {
                    continue;
                }
                if is_reported_word(&text, reported) && option_named(cmd, &text).is_none() {
                    return Some(value_tip(option, word, &text));
                }
                // A known option here is reported as a missing value.
                return None;
            }
            if !flag_shaped {
                positional_given = true;
                continue;
            }
            match option_named(cmd, &text) {
                Some((arg, expects_value)) => {
                    if expects_value {
                        pending = Some(arg);
                    }
                }
                None if is_reported_word(&text, reported) => {
                    unknown = Some((index, text));
                    break;
                }
                None => return None,
            }
        }
        let (index, text) = unknown?;
        let command = cmd.get_name();
        let long_name = |name: &str| cmd.get_arguments().any(|arg| arg.get_long() == Some(name));

        if (text == "-V" || is_reported_word(&text, "--version")) && !long_name("version") {
            return Some(format!(
                "--version (-V) is accepted only before a command: agent-coexist --version; after '{command}' it is an unknown option"
            ));
        }
        if let Some(single) = text.strip_prefix('-').filter(|rest| !rest.starts_with('-')) {
            let name = single.split_once('=').map_or(single, |(name, _)| name);
            if name.chars().count() > 1 && long_name(name) {
                return Some(format!(
                    "'{}' has one '-', so it was read as the short option {}; the long option is written with two: --{name}",
                    names::echo_raw(&text),
                    names::echo_raw(reported)
                ));
            }
        }
        if has_similar {
            return None;
        }
        let help = format!(
            "'{command}' has no option {}; 'agent-coexist {command} --help' lists its options",
            names::echo_raw(reported)
        );
        // Later words may still supply the positional: replay them.
        let positional_free = !positional_given && {
            let mut pending = false;
            let mut free = true;
            for word in &args[index + 1..] {
                let text = word.to_string_lossy();
                if text == "--" {
                    free = false;
                    break;
                }
                let flag_shaped = text.starts_with('-') && text != "-";
                if std::mem::take(&mut pending) && !flag_shaped {
                    continue;
                }
                if !flag_shaped {
                    free = false;
                    break;
                }
                pending = option_named(cmd, &text).is_some_and(|(_, expects)| expects);
            }
            free
        };
        let slot = cmd
            .get_positionals()
            .find_map(|arg| match arg.get_id().as_str() {
                "path" => Some("PATH"),
                "dir" => Some("DIR"),
                _ => None,
            });
        Some(match slot {
            Some(slot) if positional_free => format!(
                "{help}; only if '{}' is the {slot} itself, write it last, after '--'",
                names::echo_raw(&text)
            ),
            _ => help,
        })
    })
}

/// What a value option whose grammar never allows a leading `-` takes,
/// for [`value_tip`]; `None` for the free-text options, whose value may
/// start with `-` when attached with `=`.
fn dash_free_value(arg: &Arg, flag: &str) -> Option<String> {
    if arg.get_value_parser().type_id() == std::any::TypeId::of::<u64>() {
        return Some(format!(
            "a comment number made of digits only (such as {flag} 3)"
        ));
    }
    Some(match arg.get_long()? {
        "as" | "by" | "mention" | "remove" => format!("a nick: {NICK_RULE}"),
        "status" => format!("a status name: {STATUS_RULE}"),
        "section" => format!("a section name: {NICK_RULE}"),
        "type" => format!("a type: {TYPE_RULE}"),
        "since" => {
            "an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m"
                .to_string()
        }
        "for" => format!("a duration such as 1h30m: {}", time::DURATION_RULE),
        _ => return None,
    })
}

/// The tip for `word`, intended as the value of `option` but read as an
/// option because it starts with `-`. Only free-text values are told to
/// attach with `=`; for every other option a leading `-` stays invalid.
/// A value that is not valid Unicode is told to re-encode first, then
/// attach the Unicode text with `=`; the original bytes are not a working
/// `{flag}=` form. A value holding a character that [`names::echo_raw`]
/// spells out is never printed raw: a one-line command would hide it,
/// reorder the line or (an escape sequence) be rewritten by the terminal,
/// so the repair is stated with the value as the error shows it.
fn value_tip(option: &Arg, word: &OsStr, text: &str) -> String {
    let flag = option_flag(option);
    if let Some(takes) = dash_free_value(option, &flag) {
        return format!(
            "'{}' after {flag} starts with '-', so it was read as an option; {flag} takes {takes}, which never starts with '-'",
            names::echo_raw(text)
        );
    }
    if word.to_str().is_none() {
        return format!(
            "the value of {flag} is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode. A value that starts with '-' must also be attached with '=': start that same argument with '{flag}=' followed directly by the whole Unicode text"
        );
    }
    let shown = names::echo_raw(text);
    match names::shell_quote(word) {
        Some(value) if shown == text => format!(
            "the word after {flag} starts with '-', so it was read as an option; to pass it as the value of {flag}, attach it with '=': {flag}={value}"
        ),
        Some(_) => format!(
            "the word after {flag} starts with '-', so it was read as an option; to pass it as the value of {flag}, start that same argument with '{flag}=' followed directly by the whole text '{shown}', typing each <U+XXXX> as the one character it names; it cannot be shown here as a copyable command"
        ),
        // Multi-line text has no one-line shell form, so the repair is
        // stated instead of a placeholder.
        None => format!(
            "the word after {flag} starts with '-', so it was read as an option; to pass it as the value of {flag}, start that same argument with '{flag}=' followed directly by the whole text, line breaks included; it cannot be shown here as a one-line command"
        ),
    }
}

/// The usage line for a value that is not valid Unicode text. PATH and
/// DIR are native OS strings and never fail here; every other value must
/// be text. The first option whose value is not valid Unicode is named
/// when the grammar shows it.
fn invalid_unicode_message(words: &[OsString]) -> String {
    let subject = invalid_unicode_subject(words).unwrap_or_else(|| "an argument".to_string());
    format!(
        "{subject} is not valid Unicode text; retype it, or re-encode the text it came from as Unicode, then run the command again. Only PATH and DIR arguments accept file names that are not valid Unicode"
    )
}

/// Who holds the first non-Unicode word clap rejects: an option value
/// (separate, `--long=value` or attached `-mvalue`) or the guide topic,
/// also when it follows `--`. Non-Unicode PATH/DIR words are skipped,
/// since clap accepts them; an option name that is not valid Unicode is
/// an unknown argument, never this error.
fn invalid_unicode_subject(words: &[OsString]) -> Option<String> {
    with_subcommand(words, |cmd| {
        let mut pending: Option<&Arg> = None;
        let mut positional_only = false;
        for word in words.get(2..)? {
            let text = word.to_string_lossy();
            let invalid = word.to_str().is_none();
            if let Some(option) = pending.take() {
                if invalid {
                    return Some(format!("the value of {}", option_flag(option)));
                }
                continue;
            }
            if !positional_only && text == "--" {
                positional_only = true;
                continue;
            }
            if !positional_only && text.starts_with('-') && text != "-" {
                match option_named(cmd, &text) {
                    Some((arg, true)) => pending = Some(arg),
                    Some((arg, false)) if invalid => {
                        return Some(format!("the value of {}", option_flag(arg)));
                    }
                    _ => {}
                }
            } else if invalid && cmd.get_name() == "guide" {
                return Some("the guide topic".to_string());
            }
        }
        None
    })
}

const TOP_AFTER_HELP: &str = concat!(
    "\
New here? 'agent-coexist guide' explains when and how to use coexist. Exact flags for one command: 'agent-coexist <command> --help'.

Arguments:
  PATH      a collaboration file; the '.agent-besties.md' suffix may be omitted
  DIR       a directory (default: the current directory); scans are recursive
  TIME      an RFC 3339 time, e.g. 2026-09-17T09:12:00Z; any offset is converted to UTC and fractional seconds are dropped
  DURATION  e.g. 1h30m: ",
    coexist_core::duration_rule!(),
    "
  types     post, brief, agent-profile (alias: profile), state
  values    a text value (-m, --keyword, --entry, --target, --regex, --include, --exclude) that starts with '-' must be attached with '=', as in -m='- item' or --keyword=-x; a separate word starting with '-' is read as an option. Nicks, status and section names, types, times, durations and comment numbers never start with '-'

Which read command:
  check        one file: format, diagnostics and a preview; no brief target text
  read-brief   a brief with its target text
  check-state  state declarations for a directory, its ancestors and its subtree

Exit codes of the business commands (all except guide and help):
  0    first line 'ok' or 'warn'; 'warn' also exits 0, so read the first line and rows for conflicts and incomplete scans
  1    first line 'error'; also when a command could not write its report to stdout (a write command may already have changed the file: check it before retrying)
  2    usage error: one 'error: ...' line on stderr, nothing on stdout
  3    busy or locked (E-LOCKED, E-INUSE, E-FLOCKED); the report's 'fix' row says what to do next
  130  wait, wait-post or wait-state cancelled with Ctrl-C; no stdout
guide, help, --help and --version print information with no ok/warn/error line and exit 0; usage errors still exit 2, and a failed guide write exits 1.

Scan progress ('scanning dirs: N, files: M...') appears on stderr only when stderr is a terminal. No progress line does not mean success or failure; judge the result by the report and the exit code.
Examples and 'fix' commands use POSIX shell quoting (Git Bash on Windows); adjust the quoting for cmd.exe or PowerShell."
);

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
Prints one built-in Markdown page to stdout. Without a topic it prints the index page: what to do when you arrive in a project, and a first exchange between two agents.
Topics: start (which file type to use and where), read (find and check what others left), write (posts, briefs, profiles and state others can act on), wait (catch up, then wait for a reply or change).
Topics are exact lowercase words. Pages are static text; the command does not read your project.
Example: agent-coexist guide start";

const H_LIST: &str = "\
Lists the collaboration files under DIR as a tree with a type column.
--type keeps only the given types (repeatable: post, brief, agent-profile, state; 'profile' is an alias).
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Type column 'busy': the file lock or a stable read was not available within 100ms; retry. On Windows it can also mean another program holds the file open without sharing; close that program, then retry.
Type column '?': unreadable (permissions, not UTF-8, invalid H1, or over 16 MiB).
'busy' and '?' files still show under any --type and make the first line 'warn'; the command does not fail on them.
Example: agent-coexist list docs --type post --type brief";

const H_CHECK: &str = "\
Reports one file: its 'file' and 'type' rows, diagnostics in line order, then a preview (topic and latest comment for posts, entry states for briefs, sections for profiles, declarations for state).
Brief targets are checked but not printed; use read-brief for their text. For every declaration that applies to a directory, use check-state.
Waits up to 1s for a shared lock; if the file stays busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist check docs/standup";

const H_PROFILE: &str = "\
--section <name> -m <content> sets that section's body, stored verbatim in a code fence; --section <name> --remove deletes the section. Without --section, creates the file (heading only) or reports 'unchanged'. Content that starts with '-' must be attached: -m='- item'.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist profile agents/alice --section role -m 'Rust implementer. Owns crates/core.'";

const H_POST: &str = "\
-m is the comment body, stored verbatim in a code fence. In it, @nick mentions someone and @#n replies to comment n. A body that starts with '-', such as a Markdown list, must be attached: -m='- item'. --as is the author nick written in the comment heading.
--edit <n> replaces the whole body of your own comment #n (its author must be --as), so resend everything you want to keep.
The report's 'comment' row names the comment written.
Waits up to 5s for the file lock (AGENT_COEXIST_LOCK_WAIT_MS overrides); if still busy: E-LOCKED, exit 3; retry the same command shortly.
Example: agent-coexist post docs/standup --as bob -m '@#1 parser done, see src/parser/mod.rs'";

const H_SUBSCRIBE: &str = "\
Adds --as to the post's subscriber list, so read-notification --as <nick> lists every new comment in that post. Nothing is pushed to subscribers, and wait-post does not use the list.
--cancel removes --as from the list. Subscribing twice, or cancelling a nick that is not subscribed, changes nothing ('unchanged <nick>').
Locking is as for post: waits up to 5s (AGENT_COEXIST_LOCK_WAIT_MS overrides), then E-LOCKED, exit 3.
Example: agent-coexist subscribe docs/standup --as alice";

const H_BRIEF: &str = "\
The usage forms, in order: create the brief (or report 'unchanged'); set the description; add or update an entry for a target; replace one entry's note; remove an entry; refresh hashes (every entry, or only --entry). Other combinations are rejected.
-m is the description without --target/--entry, and the entry's note with them. Text that starts with '-' must be attached: -m='- item', and likewise --regex='-x'.
--target is relative to the brief's directory, not the current directory. Without --entry, the title is the target's file name.
--regex: when the brief is checked or read, the target must still match it, or the entry is 'stale'; writing only checks that it compiles. --regex '' deletes the entry's regex lines. Use \\r or \\n in a pattern to match line breaks; --regex and --target cannot contain raw line breaks.
--refresh accepts the targets' current bytes: read what changed and update the note first.
--as is your nick. It is not written; it only exempts your own 'locked' declarations. A valid 'locked' by another nick on the file's directory or an ancestor refuses the write with E-FLOCKED (exit 3); without --as, your own 'locked' refuses it too.
If another write holds the file, fails after about 200ms with E-LOCKED (exit 3); retry shortly. On E-FLOCKED, retry after the time shown in 'reason' or ask the holder; do not retry in a loop.
Example: agent-coexist brief docs/onboarding --target ../src/parser/mod.rs --entry 'parser entry point'";

const H_STATE: &str = "\
Declare: --as <nick> --status <name> records that <nick> holds <name> on DIR and everything below it until the end time. Declaring again replaces your entries under that status with one from now, except an entry on a line that also has unparseable fragments (W-STATE-ENTRY): that line is kept, and the latest end time still wins; check-state shows the result.
Remove: --remove <nick> removes that nick's declarations in DIR. Add --status to remove only that status; without it every status goes, including a live 'locked'.
Built-in statuses: 'locked' = please do not change this directory or its subdirectories until the end time; 'observing' = I am watching, changes are welcome. Other names are stored and shown but have no built-in meaning.
--for defaults to 2h for 'locked' and 8h otherwise. In the remove form --as defaults to the removed nick; removing another nick's declaration adds a 'removed-other' row and a 'warn' first line.
DIR must already exist; it is never created.
'locked' is a request between agents, not access control, and another nick's 'locked' never blocks a state write. Locks overlapping yours are listed as 'conflict' rows with a 'warn' first line (exit 0); talk to the holder in a post.
The conflict check reads the state files of DIR and its ancestors and the state files below DIR that ignore rules do not skip; state has no --no-ignore, so to see locks inside ignored directories run check-state DIR --no-ignore.
If another write holds the state file, fails after about 200ms with E-LOCKED (exit 3); retry shortly.
Example: agent-coexist state src --as alice --status locked --for 1h30m";

const H_READ_NOTIFICATION: &str = "\
Lists, oldest first, what happened since --since that concerns the --as nick: comments that mention it, reply to its comments, or appear in posts it subscribes to; state declarations by others; locks that overlap its own; its own declarations that end soon or have ended.
Scans every post and state file under DIR that ignore rules do not skip (default: the current directory) and also reads ancestor state files. It never writes files and keeps no read cursor; for files inside ignored directories add --no-ignore.
--since is an RFC 3339 time (e.g. 2026-09-17T09:12:00Z) or a duration back from now (e.g. 1h30m). The window ends before the current second, so a comment written this second shows up in a later scan.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each file waits at most 100ms for a shared lock. Busy or unparseable files become 'skipped' or 'partial' rows with a 'warn' first line; read them directly before concluding nothing is new.
Example: agent-coexist read-notification --as alice --since 1h";

const H_LOOKFOR: &str = "\
Lists matching comments with line, number, author, time and an excerpt. Without filters it lists every comment; with several, a comment must match all of them.
--keyword is a case-insensitive substring of the raw body or heading line, matched exactly as typed (spaces are not folded).
--by is the comment's author (check lists a post's 'authors'). --mention matches @nick in the body, --reply-to matches @#n in the body, and --comment selects comment #n.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist lookfor-comment docs/standup --by bob";

const H_READ_BRIEF: &str = "\
Prints the description, then each entry's state, note and target text. With a regex, only the lines of its first match are printed; without one, the whole target.
An entry with no path, a missing or unreadable target, a failing regex, or a target over 64 MiB prints no text; its row says why.
--entry prints only that entry. For entry states without target text, use check.
Waits up to 1s for a shared lock; if still busy: E-LOCKED, exit 3; retry shortly.
Example: agent-coexist read-brief docs/onboarding --entry 'parser entry point'";

const H_CHECK_STATE: &str = "\
Shows the declarations that apply to DIR: 'inherited' rows from ancestor directories (valid ones only), DIR's own declarations, then one group per state file below DIR that ignore rules do not skip (add --no-ignore to see declarations in skipped directories).
A row ends with 'conflict' when it is a valid 'locked' and another nick holds a valid 'locked' shown in this same report on an overlapping directory (the same, an ancestor or a descendant; both rows are marked), or with 'expired' when a declaration in DIR or below has ended. Either mark makes the first line 'warn' (exit 0). Only this report's declarations are compared: an inherited lock can still conflict with a lock in another branch below that ancestor; check-state on the ancestor shows it.
--include/--exclude are gitignore-style globs relative to DIR. --include does not reach into directories that ignore rules skip; add --no-ignore for that.
Each state file waits at most 100ms for a shared lock. Busy or unparseable ones become 'skipped' or 'partial' rows with a 'warn' first line.
For one state file with its diagnostics, use check <DIR>/state.
Example: agent-coexist check-state src";

const H_WAIT: &str = "\
Waits for the next change to this collaboration file (any type), prints one report of what changed, and exits 0. It never writes.
Once it has recorded its starting point, it writes one line 'waiting  wait  <object>' to stderr. A change made before that line is not reported, so read the file again after the line appears.
The file is sampled about once a second. A missing file is a valid start: the wait reports its creation (its directory must exist), so a mistyped path waits forever.
Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3); on Windows, a program holding the file open without sharing ends it with E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: read the file again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait docs/onboarding";

const H_WAIT_POST: &str = "\
Waits for a comment in this post that mentions @<nick> or replies to a comment by <nick> and is new, edited or newly relevant; prints those comments and exits 0. With --as, subscriber changes and deleted comments do not count; without --as, any change to the post counts.
Once it has recorded its starting point, it writes one line 'waiting  wait-post  <object>' to stderr. A comment written before that line is not reported, so read the post again after the line appears.
The post is sampled about once a second; a missing post is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy post, an unreadable post, or an invalid sample — a restarted wait starts from a new baseline: read the post again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
Example: agent-coexist wait-post docs/standup --as alice";

const H_WAIT_STATE: &str = "\
Watches only DIR/state.agent-besties.md (DIR defaults to the current directory), not ancestors or subdirectories.
--status (locked, observing or another status name) waits for a declaration under that status to be added, removed or renewed; without it, any change to the file counts. A declaration that simply expires does not count; use check-state for current declarations.
Once it has recorded its starting point, it writes one line 'waiting  wait-state  <object>' to stderr. A change made before that line is not reported, so run check-state again after the line appears.
The file is sampled about once a second; a missing state file is a valid start. Each sample waits up to 1s for a shared lock. If the file stays busy, the wait ends with E-LOCKED (exit 3), or on Windows E-INUSE (exit 3). However a wait ends — a busy file, an unreadable file, or an invalid sample — a restarted wait starts from a new baseline: run check-state on the directory again after its ready line.
There is no timeout. Ctrl-C cancels: exit 130, no stdout.
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

    fn words(args: &[&str]) -> Vec<OsString> {
        std::iter::once("agent-coexist")
            .chain(args.iter().copied())
            .map(OsString::from)
            .collect()
    }

    /// A hyphen-leading value after a text option gets a tip whose form
    /// really passes it as that option's value (clap's `-- <word>` would
    /// make it the PATH); a negative comment number is not told to attach,
    /// since it stays invalid either way.
    #[test]
    fn hyphen_value_tip_recovers_the_intended_value() {
        let args = ["post", "standup", "--as", "alice", "-m", "- item"];
        let message = grammar_error_message(parse_args(&args).unwrap_err(), &words(&args));
        assert!(message.contains("-m='- item'"), "{message}");
        assert!(!message.contains("use '--"), "{message}");
        match parse_args(&["post", "standup", "--as", "alice", "-m=- item"]) {
            Ok(Cli {
                command: TopCommand::Business(Commands::Post { message, path, .. }),
            }) => {
                assert_eq!(message, "- item");
                assert_eq!(path, PathBuf::from("standup"));
            }
            other => panic!("attached form must parse as the -m value, got {other:?}"),
        }

        let args = [
            "post", "standup", "--as", "alice", "-m", "x", "--edit", "-1",
        ];
        let message = grammar_error_message(parse_args(&args).unwrap_err(), &words(&args));
        assert!(
            !message.contains("--edit=-1") && !message.contains("use '--"),
            "{message}"
        );
    }

    /// A tip never carries a bidi override or escape the user typed: such
    /// a value is not offered as a copyable `-m=` command, and an unknown
    /// option is echoed in `<U+XXXX>` form.
    #[test]
    fn grammar_tips_never_echo_raw_controls() {
        let cases: [&[&str]; 3] = [
            &["post", "standup", "--as", "alice", "-m", "-\u{202E}item"],
            &["post", "standup", "--as", "alice", "-m", "-\u{1B}[31mitem"],
            &[
                "post",
                "standup",
                "--bad\u{202E}",
                "-m",
                "hello",
                "--as",
                "alice",
            ],
        ];
        for args in cases {
            let message = grammar_error_message(parse_args(args).unwrap_err(), &words(args));
            assert!(!message.contains(['\u{202E}', '\u{1B}']), "{message:?}");
            assert!(!message.contains("-m='-"), "{message:?}");
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

    fn resolve_args(args: &[&str]) -> Result<Invocation, String> {
        let ctx = Ctx::new(Some(String::new()), None).expect("ctx");
        match parse_args(args).expect("grammar") {
            Cli {
                command: TopCommand::Business(command),
            } => resolve(command, &ctx),
            other => panic!("expected a business command, got {other:?}"),
        }
    }

    /// A nick, status, section or entry title rejected for a control
    /// character or a double space shows the value with the control
    /// visible. The trim repair is offered only when it makes the value
    /// valid, and applying it runs the original command (regression: a CR
    /// from a CRLF script, or two spaces at the end of a title, used to be
    /// rejected with no echo and no repair).
    #[test]
    fn plain_value_rejection_echoes_value_and_repair_resolves() {
        let cases: [(&[&str], usize, Option<&str>); 11] = [
            (
                &["post", "standup", "--as", "alice\r", "-m", "hi"],
                3,
                Some("alice"),
            ),
            (
                &["post", "standup", "--as", "1024\r", "-m", "hi"],
                3,
                Some("1024"),
            ),
            (
                &["state", "--as", "alice", "--status", "locked\r"],
                4,
                Some("locked"),
            ),
            (
                &["profile", "card", "--section", "role  ", "-m", "x"],
                3,
                Some("role"),
            ),
            (
                &["read-notification", "--as", "\talice", "--since", "1h"],
                2,
                Some("alice"),
            ),
            (&["post", "standup", "--as", "ali\tce", "-m", "hi"], 3, None),
            (&["state", "--as", "alice", "--status", "lo  cked"], 4, None),
            (
                &[
                    "read-brief",
                    "onboarding",
                    "--entry",
                    "parser entry point\r",
                ],
                3,
                Some("parser entry point"),
            ),
            (
                &[
                    "brief",
                    "onboarding",
                    "--entry",
                    "parser entry point  ",
                    "--refresh",
                ],
                3,
                Some("parser entry point"),
            ),
            (
                &[
                    "read-brief",
                    "onboarding",
                    "--entry",
                    "parser  entry point ",
                ],
                3,
                None,
            ),
            (
                &[
                    "read-brief",
                    "onboarding",
                    "--entry",
                    "\u{1}parser entry point",
                ],
                3,
                None,
            ),
        ];
        for (args, at, repaired) in cases {
            let message = resolve_args(args).err().expect("rejected");
            assert!(
                message.contains(&format!("(got '{}')", names::echo_raw(args[at]))),
                "{message}"
            );
            let fix = message
                .split_once("before or after '")
                .map(|(_, rest)| rest.strip_suffix('\'').expect("repair ends the message"));
            assert_eq!(fix, repaired, "{message}");
            if let Some(fix) = fix {
                let mut retry = args.to_vec();
                retry[at] = fix;
                assert!(resolve_args(&retry).is_ok(), "{retry:?}");
            }
        }
    }

    /// A raw CR/LF in `--target`/`--regex` (a CRLF script's leftover) is
    /// a usage rejection, the trimmed target and the escaped pattern are
    /// accepted retries, and the one-line error never carries the raw
    /// control character itself.
    #[test]
    fn target_and_regex_newline_rejections_resolve_when_corrected() {
        let rejected: [&[&str]; 3] = [
            &["brief", "onboarding", "--target", "src/main.rs\r"],
            &["brief", "onboarding", "--target", "src/ma\rin.rs"],
            &[
                "brief",
                "onboarding",
                "--target",
                "src/main.rs",
                "--regex",
                "main.*\n.*run",
            ],
        ];
        for args in rejected {
            let message = resolve_args(args).err().expect("rejected");
            assert!(
                !message.contains('\r') && !message.contains('\n'),
                "{message:?}"
            );
        }
        // The corrected retries resolve: the trimmed target, and the
        // pattern with `\n` written as the two-character escape.
        assert!(resolve_args(&["brief", "onboarding", "--target", "src/main.rs"]).is_ok());
        assert!(resolve_args(&[
            "brief",
            "onboarding",
            "--target",
            "src/main.rs",
            "--regex",
            "main.*\\n.*run",
        ])
        .is_ok());
    }
}
