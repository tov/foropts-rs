use super::super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
enum GitCmd<'a> {
    Clone(CloneCmd<'a>),
    Init(InitCmd<'a>),
    Add(AddCmd<'a>),
    Commit(CommitCmd<'a>),
    Push(PushCmd<'a>),
    Pull(PullCmd<'a>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GlobalOpts {
    version: bool,
    help: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloneCmd<'a> {
    global: GlobalOpts,
    verbose: bool,
    jobs: Option<&'a str>,
    repo: &'a str,
    dir: Option<&'a str>
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InitCmd<'a> {
    global: GlobalOpts,
    bare: bool,
    dir: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AddCmd<'a> {
    global: GlobalOpts,
    dry_run: bool,
    verbose: bool,
    interactive: bool,
    all: bool,
    files: Vec<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommitCmd<'a> {
    global: GlobalOpts,
    message: Option<&'a str>,
    all: bool,
    files: Vec<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PushCmd<'a> {
    global: GlobalOpts,
    verbose: bool,
    force: bool,
    delete: bool,
    all: bool,
    repo: Option<&'a str>,
    refspecs: Vec<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PullCmd<'a> {
    global: GlobalOpts,
    tags: bool,
    rebase: Option<&'a str>,
    repo: Option<&'a str>,
    refspecs: Vec<&'a str>,
}

fn git<'a>(args: &'a [&'a str]) -> Result<GitCmd<'a>, String> {
    let config0 = HashConfig::new()
        .opt("version", false)
        .opt("help", false);

    let mut parser = config0.parse_slice(args);
    let mut global = GlobalOpts {
        version: false,
        help: false,
    };

    while let Some(item) = parser.next() {
        match item {
            Item::Opt(flag, None) => {
                if flag.is("version") {
                    global.version = true;
                } else if flag.is("help") {
                    global.help = true;
                } else {
                    unreachable!("0");
                }
            }

            Item::Positional(command) => {
                match command {
                    "clone" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt('v', false).opt("verbose", false)
                            .opt('q', false).opt("quiet", false)
                            .opt('j', true).opt("jobs", true);

                        enum State<'a> {
                            Nothing,
                            RepoDir(&'a str, Option<&'a str>),
                        }

                        let mut verbose = false;
                        let mut jobs    = None;
                        let mut state   = State::Nothing;

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, param) => {
                                    if flag.is('v') || flag.is("verbose") {
                                        verbose = true;
                                    } else if flag.is('q') || flag.is("quiet") {
                                        verbose = false;
                                    } else if flag.is('j') || flag.is("jobs") {
                                        jobs = param;
                                    } else {
                                        unreachable!("1");
                                    }
                                }

                                Item::Positional(pos) => {
                                    match state {
                                        State::Nothing =>
                                            state = State::RepoDir(pos, None),
                                        State::RepoDir(repo, None) =>
                                            state = State::RepoDir(repo, Some(pos)),
                                        State::RepoDir(_, Some(_)) =>
                                            Err(format!("unexpected argument: {}", pos))?,
                                    }
                                }

                                Item::Error(kind) => {
                                    Err(kind.to_string())?
                                }
                            }
                        }

                        match state {
                            State::Nothing => Err("expected argument: repo")?,
                            State::RepoDir(repo, dir) => return Ok(GitCmd::Clone(CloneCmd {
                                global, verbose, jobs, repo, dir,
                            })),
                        }
                    }

                    "init" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt("bare", false);

                        let mut bare = false;
                        let mut dir = None;

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, _) => {
                                    if flag.is("bare") {
                                        bare = true;
                                    } else {
                                        Err(format!("unexpected argument: {}", item))?;
                                    }
                                },

                                Item::Positional(arg) => {
                                    if dir.is_none() {
                                        dir = Some(arg);
                                    } else {
                                        Err(format!("unexpected argument: {}", arg))?;
                                    }
                                }

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Init(InitCmd { global, bare, dir, }));
                    }

                    "add" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt('n', false).opt("dry-run", false)
                            .opt('v', false).opt("verbose", false)
                            .opt('i', false).opt("interactive", false)
                            .opt('A', false).opt("all", false);

                        let mut command = AddCmd {
                            global,
                            dry_run: false,
                            verbose: false,
                            interactive: false,
                            all: false,
                            files: Vec::new(),
                        };

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, _) => {
                                    if flag.is('n') || flag.is("dry-run") {
                                        command.dry_run = true;
                                    } else if flag.is('v') || flag.is("verbose") {
                                        command.verbose = true;
                                    } else if flag.is('i') || flag.is("interactive") {
                                        command.interactive = true;
                                    } else if flag.is('A') || flag.is("all") {
                                        command.all = true;
                                    } else {
                                        unreachable!();
                                    }
                                }

                                Item::Positional(file) => command.files.push(file),

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Add(command));
                    }

                    "commit" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt('m', true).opt("message", true)
                            .opt('a', false).opt("all", false);

                        let mut command = CommitCmd {
                            global,
                            message: None,
                            all: false,
                            files: Vec::new(),
                        };

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, param) => {
                                    if flag.is('m') || flag.is("message") {
                                        command.message = param;
                                    } else if flag.is('a') || flag.is("all") {
                                        command.all = true;
                                    } else {
                                        unreachable!();
                                    }
                                }

                                Item::Positional(file) => command.files.push(file),

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Commit(command));
                    }

                    "push" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt('v', false).opt("verbose", false)
                            .opt('q', false).opt("quiet", false)
                            .opt('f', false).opt("force", false)
                            .opt('d', false).opt("delete", false)
                            .opt("all", false)
                            .opt("repo", true);

                        let mut command = PushCmd {
                            global,
                            verbose: false,
                            force: false,
                            delete: false,
                            all: false,
                            repo: None,
                            refspecs: Vec::new(),
                        };

                        let mut positional_repo = false;

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, param) => {
                                    if flag.is('v') || flag.is("verbose") {
                                        command.verbose = true;
                                    } else if flag.is('q') || flag.is("quiet") {
                                        command.verbose = false;
                                    } else if flag.is('f') || flag.is("force") {
                                        command.force = false;
                                    } else if flag.is('d') || flag.is("delete") {
                                        command.delete = false;
                                    } else if flag.is('a') || flag.is("all") {
                                        command.all = true;
                                    } else if flag.is("repo") {
                                        if !positional_repo {
                                            command.repo = param;
                                        }
                                    } else {
                                        unreachable!();
                                    }
                                }

                                Item::Positional(file) => {
                                    if positional_repo {
                                        command.refspecs.push(file);
                                    } else {
                                        command.repo = Some(file);
                                        positional_repo = true;
                                    }
                                },

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Push(command));
                    }

                    "pull" => {
                        *parser.config_mut() = HashConfig::new()
                            .opt('t', false).opt("tags", false)
                            .opt('r', Presence::IfAttached)
                            .opt("rebase", Presence::IfAttached);

                        let mut command = PullCmd {
                            global,
                            tags: false,
                            rebase: None,
                            repo: None,
                            refspecs: Vec::new(),
                        };

                        let mut positional_repo = false;

                        while let Some(item) = parser.next() {
                            match item {
                                Item::Opt(flag, param) => {
                                    if flag.is('t') || flag.is("tags") {
                                        command.tags = true;
                                    } else if flag.is('r') || flag.is("rebase") {
                                        command.rebase = param;
                                    } else {
                                        unreachable!();
                                    }
                                }

                                Item::Positional(file) => {
                                    if positional_repo {
                                        command.refspecs.push(file);
                                    } else {
                                        command.repo = Some(file);
                                        positional_repo = true;
                                    }
                                },

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Pull(command));
                    }

                    _ => Err(format!("unknown command: {}", command))?,
                }
            }

            Item::Error(kind) => Err(kind.to_string())?,

            item => Err(format!("unexpected argument: {}", item))?,
        }
    }

    Err("no command")?
}
