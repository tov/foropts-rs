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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    let mut parser = config0.into_slice_iter(args);
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

                        let mut command = CloneCmd {
                            global,
                            verbose: false,
                            jobs:    None,
                            repo:    "",
                            dir:     None,
                        };
                        let mut repo_set = false;

                        for item in parser {
                            match item {
                                Item::Opt(flag, param) => {
                                    if flag.is('v') || flag.is("verbose") {
                                        command.verbose = true;
                                    } else if flag.is('q') || flag.is("quiet") {
                                        command.verbose = false;
                                    } else if flag.is('j') || flag.is("jobs") {
                                        command.jobs = param;
                                    } else {
                                        unreachable!("1");
                                    }
                                }

                                Item::Positional(pos) => {
                                    if !repo_set {
                                        command.repo = pos;
                                        repo_set    = true;
                                    } else if command.dir.is_none() {
                                        command.dir = Some(pos);
                                    } else {
                                        Err(format!("unexpected argument: {}", pos))?;
                                    }
                                }

                                Item::Error(kind) => {
                                    Err(kind.to_string())?
                                }
                            }
                        }

                        if !repo_set {
                            return Err("expected argument: repo".to_owned());
                        }

                        return Ok(GitCmd::Clone(command));
                    }

                    "init" => {
                        let init_config: &'static [_] = &[(Flag::Long("bare"), false)];
                        let mut init_parser = parser.with_config(init_config);

                        let mut result = InitCmd {
                            global,
                            bare: false,
                            dir:  None,
                        };

                        while let Some(item) = init_parser.next() {
                            match item {
                                Item::Opt(flag, _) => {
                                    if flag.is("bare") {
                                        result.bare = true;
                                    } else {
                                        unreachable!("2");
                                    }
                                },

                                Item::Positional(arg) => {
                                    if result.dir.is_none() {
                                        result.dir = Some(arg);
                                    } else {
                                        Err(format!("unexpected argument: {}", arg))?;
                                    }
                                }

                                Item::Error(kind) => Err(kind.to_string())?,
                            }
                        }

                        return Ok(GitCmd::Init(result));
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
                                        unreachable!("3");
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
                                        unreachable!("4");
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
                                        if positional_repo {
                                            Err("repo already given")?
                                        } else {
                                            command.repo = param;
                                        }
                                    } else {
                                        unreachable!("5");
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
                                        unreachable!("6");
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

macro_rules! err {
    ( $fmt:expr $( , $arg:expr )* ) => {
        Err(format!($fmt $( , $arg )* ))
    };
}

#[test]
fn no_command_tests() {
    assert_eq!( git(&[]),
                err!("no command") );
    assert_eq!( git(&["--version"]),
                err!("no command") );
    assert_eq!( git(&["--version", "--help"]),
                err!("no command") );
    assert_eq!( git(&["-version"]),
                err!("unknown flag: -v") );
}


const GLOBAL: GlobalOpts = GlobalOpts {
    version: false,
    help: false,
};

#[test]
fn clone_tests() {
    let ok     = |cmd| Ok(GitCmd::Clone(cmd));
    let global = GLOBAL;

    assert_eq!( git(&["clone"]),
                err!("expected argument: repo") );
    assert_eq!( git(&["clone", "REPO"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: None, repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["--help", "clone", "REPO"]),
                ok(CloneCmd {
                    global: GlobalOpts { version: false, help: true, },
                    verbose: false, jobs: None, repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "--help", "REPO"]),
                err!("unknown flag: --help") );
    assert_eq!( git(&["clone", "-v", "REPO"]),
                ok(CloneCmd {
                    global, verbose: true, jobs: None, repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "-v", "REPO", "-q"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: None, repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "REPO", "--jobs", "4"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: Some("4"), repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "REPO", "--jobs=4"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: Some("4"), repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "REPO", "-j4"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: Some("4"), repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "REPO", "-j", "4"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: Some("4"), repo: "REPO", dir: None,
                }) );
    assert_eq!( git(&["clone", "REPO", "DIR", "-vqj", "4"]),
                ok(CloneCmd {
                    global, verbose: false, jobs: Some("4"), repo: "REPO", dir: Some("DIR"),
                }) );
    assert_eq!( git(&["clone", "REPO", "DIR", "EXTRA", "-vqj", "4"]),
                err!("unexpected argument: EXTRA") );
    assert_eq!( git(&["clone", "REPO", "DIR", "-vQj", "4"]),
                err!("unknown flag: -Q") );
}

#[test]
fn init_tests() {
    let ok     = |cmd| Ok(GitCmd::Init(cmd));
    let global = GLOBAL;

    assert_eq!( git(&["init"]),
                ok(InitCmd { global, dir: None, bare: false, }) );
    assert_eq!( git(&["init", "--help"]),
                err!("unknown flag: --help") );
    assert_eq!( git(&["init", "somewhere"]),
                ok(InitCmd { global, dir: Some("somewhere"), bare: false, }) );
    assert_eq!( git(&["init", "--bare", "somewhere"]),
                ok(InitCmd { global, dir: Some("somewhere"), bare: true, }) );
    assert_eq!( git(&["init", "somewhere", "--bare"]),
                ok(InitCmd { global, dir: Some("somewhere"), bare: true, }) );
    assert_eq!( git(&["init", "somewhere", "else"]),
                err!("unexpected argument: else") );
}

#[test]
fn add_tests() {
    let ok     = |cmd| Ok(GitCmd::Add(cmd));
    let global = GLOBAL;

    assert_eq!( git(&["add"]),
                ok(AddCmd {
                    global, dry_run: false, verbose: false,
                    interactive: false, all: false, files: vec![],
                }) );
    assert_eq!( git(&["add", "--dry-run"]),
                ok(AddCmd {
                    global, dry_run: true, verbose: false,
                    interactive: false, all: false, files: vec![],
                }) );
    assert_eq!( git(&["add", "-n"]),
                ok(AddCmd {
                    global, dry_run: true, verbose: false,
                    interactive: false, all: false, files: vec![],
                }) );
    assert_eq!( git(&["add", "-n", "foo"]),
                ok(AddCmd {
                    global, dry_run: true, verbose: false,
                    interactive: false, all: false, files: vec!["foo"],
                }) );
    assert_eq!( git(&["add", "-n", "foo", "bar"]),
                ok(AddCmd {
                    global, dry_run: true, verbose: false,
                    interactive: false, all: false, files: vec!["foo", "bar"],
                }) );
    assert_eq!( git(&["add", "-n", "foo", "bar", "baz"]),
                ok(AddCmd {
                    global, dry_run: true, verbose: false,
                    interactive: false, all: false, files: vec!["foo", "bar", "baz"],
                }) );
}

#[test]
fn push_tests() {
    let ok     = |cmd| Ok(GitCmd::Push(cmd));
    let global = GLOBAL;

    assert_eq!( git(&["push"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: None, refspecs: vec![],
                }) );
    assert_eq!( git(&["push", "a_repo", "a_refspec"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("a_repo"), refspecs: vec!["a_refspec"],
                }) );
    assert_eq!( git(&["push", "a_repo", "a_refspec", "another"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("a_repo"), refspecs: vec!["a_refspec", "another"],
                }) );
    assert_eq!( git(&["push", "--repo", "flag_repo"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("flag_repo"), refspecs: vec![],
                }) );
    assert_eq!( git(&["push", "--repo", "flag_repo", "--repo", "afr"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("afr"), refspecs: vec![],
                }) );
    assert_eq!( git(&["push", "--repo", "flag_repo", "--repo", "afr", "a_refspec"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("a_refspec"), refspecs: vec![],
                }) );
    assert_eq!( git(&["push", "--repo", "flag_repo", "a_repo", "a_refspec"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("a_repo"), refspecs: vec!["a_refspec"],
                }) );
    assert_eq!( git(&["push", "--repo", "flag_repo", "a_repo", "a_refspec", "another"]),
                ok(PushCmd {
                    global, verbose: false, force: false, delete: false, all: false,
                    repo: Some("a_repo"), refspecs: vec!["a_refspec", "another"],
                }) );
    assert_eq!( git(&["push", "a_repo", "--repo", "flag_repo", "a_refspec", "another"]),
                err!("repo already given") );
}

#[test]
fn pull_tests() {
    let ok     = |cmd| Ok(GitCmd::Pull(cmd));
    let global = GLOBAL;

    assert_eq!( git(&["pull"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: None,
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "-r"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: None,
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "-rpreserve"]),
                ok(PullCmd {
                    global, tags: false, rebase: Some("preserve"), repo: None,
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "-r", "preserve"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: Some("preserve"),
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "--rebase=preserve"]),
                ok(PullCmd {
                    global, tags: false, rebase: Some("preserve"), repo: None,
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "moo", "--rebase", "preserve"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: Some("moo"),
                    refspecs: vec!["preserve"],
                }) );
    assert_eq!( git(&["pull", "--rebase=", "preserve"]),
                ok(PullCmd {
                    global, tags: false, rebase: Some(""), repo: Some("preserve"),
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "one"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: Some("one"),
                    refspecs: vec![],
                }) );
    assert_eq!( git(&["pull", "one", "two"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: Some("one"),
                    refspecs: vec!["two"],
                }) );
    assert_eq!( git(&["pull", "one", "two", "three"]),
                ok(PullCmd {
                    global, tags: false, rebase: None, repo: Some("one"),
                    refspecs: vec!["two", "three"],
                }) );
}
