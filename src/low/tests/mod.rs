use super::*;

mod git_example;
mod helpers;

use self::helpers::*;

fn _long_short_owned() -> HashConfig<String> {
    TokenHashConfig::new()
        .short('a', false)
        .long("all", false)
        .short('o', true)
        .long("out", true)
}

fn _long_short_ref() -> HashConfig<&'static str> {
    TokenHashConfig::new()
        .short('a', false)
        .long("all", false)
        .short('o', true)
        .long("out", true)
}

#[test]
fn owned() {
    let config = HashConfig::new()
        .opt('a', false)
        .opt("all".to_owned(), false)
        .opt('o', true)
        .opt("out".to_owned(), true);

    let result: Vec<_> = config.into_slice_iter(&["-a", "-ofile"]).collect();

    assert_eq!( result,
                &[ Item::Opt(Flag::Short('a'), None, ()),
                   Item::Opt(Flag::Short('o'), Some("file"), ()) ] );
}

#[test]
fn borrowed_short() {
    assert_parse( &["-a", "-ofile"],
                  &[ flag('a'), opt_with('o', "file") ]);
    assert_parse( &["-aofile"],
                  &[ flag('a'), opt_with('o', "file") ] );
    assert_parse( &["-oafile"],
                  &[ opt_with('o', "afile") ] );
    assert_parse( &["-o", "afile"],
                  &[ opt_with('o', "afile") ] );
    assert_parse( &["-o", "a", "file"],
                  &[ opt_with('o', "a"), pos("file") ] );
    assert_parse( &["-eieio"],
                  &[ unknown('e'),
                      unknown('i'),
                      unknown('e'),
                      unknown('i'),
                      missing_param('o'),
                  ] );
    assert_parse( &["-e-o", "-a", "--", "-a"],
                  &[ unknown('e'),
                      unknown('-'),
                      opt_with('o', "-a"),
                      pos("-a"),
                  ] );
}

#[test]
fn borrowed_long() {
    assert_parse( &["--all", "--out=file"],
                  &[ flag("all"), opt_with("out", "file") ] );
    assert_parse( &["--all", "--out", "file"],
                  &[ flag("all"), opt_with("out", "file") ] );
    assert_parse( &["--out", "-afile"],
                  &[ opt_with("out", "-afile") ] );
    assert_parse( &["--out"],
                  &[ missing_param("out") ] );
    assert_parse( &["--all=none"],
                  &[ unexpected_param("all", "none") ] )
}



// Testing helper

fn assert_parse(input: &[&str], output: &[Item<()>]) {
    let config = HashConfig::new()
        .opt('a', false)
        .opt("all", false)
        .opt('o', true)
        .opt("out", true);

    let result: Vec<_> = config.slice_iter(input).collect();
    assert_eq!( result, output );
}
