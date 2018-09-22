use super::slice_iter::SliceIter;
use super::flag::Flag;
use super::policy::{OptPolicy, Presence};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait Config {
    type Token;

    fn get_short_policy(&self, short: char) -> Option<OptPolicy<Self::Token>>;

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<Self::Token>>;

    fn slice_iter<'a, Arg>(&self, args: &'a [Arg]) -> SliceIter<'a, &Self, Arg>
        where Arg: Borrow<str> {
        
        SliceIter::new(self, args)
    }

    fn into_slice_iter<Arg>(self, args: &[Arg]) -> SliceIter<Self, Arg>
        where Self: Sized,
              Arg:  Borrow<str> {

        SliceIter::new(self, args)
    }
}

impl<'a, T: Config + ?Sized> Config for &'a T {
    type Token = T::Token;

    fn get_short_policy(&self, short: char) -> Option<OptPolicy<T::Token>> {
        T::get_short_policy(self, short)
    }

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<T::Token>> {
        T::get_long_policy(self, long)
    }
}

impl<T: Config + ?Sized> Config for Box<T> {
    type Token = T::Token;

    fn get_short_policy(&self, short: char) -> Option<OptPolicy<T::Token>> {
        T::get_short_policy(&self, short)
    }

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<T::Token>> {
        T::get_long_policy(&self, long)
    }
}

/// The configuration for the argument parser.
#[derive(Clone)]
pub struct HashConfig<L, P = ()> {
    short_opts: HashMap<char, OptPolicy<T>>,
    long_opts:  HashMap<L, OptPolicy<T>>,
}

impl<L, T> fmt::Debug for HashConfig<L, T>
    where L: Eq + Hash + Borrow<str>,
          T: fmt::Debug {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut map = f.debug_map();

        for (key, value) in &self.short_opts {
            map.entry(&Flag::Short::<()>(*key), value);
        }

        for (key, value) in &self.long_opts {
            map.entry(&Flag::Long(key.borrow()), value);
        }

        map.finish()
    }
}

impl<L, T> Config for HashConfig<L, T>
    where L: Eq + Hash + Borrow<str>,
          T: Clone {

    type Token = T;

    fn get_short_policy(&self, short: char) -> Option<OptPolicy<T>> {
        self.short_opts.get(&short).cloned()
    }

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<T>> {
        self.long_opts.get(long).cloned()
    }
}

impl<L, T> HashConfig<L, T>
    where L: Eq + Hash + Borrow<str> {

    pub fn new() -> Self {
        HashConfig {
            short_opts: HashMap::new(),
            long_opts:  HashMap::new(),
        }
    }

    pub fn with_capacities(shorts: usize, longs: usize) -> Self {
        HashConfig {
            short_opts: HashMap::with_capacity(shorts),
            long_opts:  HashMap::with_capacity(longs),
        }
    }

    pub fn opt<F, P>(self, flag: F, param: P) -> Self
        where F: Into<Flag<L>>,
              P: Into<OptPolicy<T>> {

        match flag.into() {
            Flag::Short(short) => self.short(short, param),
            Flag::Long(long)   => self.long(long, param),
        }
    }

    pub fn short<P>(mut self, flag: char, param: P) -> Self
        where P: Into<OptPolicy<T>> {

        self.short_opts.insert(flag, param.into());
        self
    }

    pub fn long<S, P>(mut self, flag: S, param: P) -> Self
        where S: Into<L>,
              P: Into<OptPolicy<T>> {

        self.long_opts.insert(flag.into(), param.into());
        self
    }

    pub fn both<S, P>(self, short: char, long: S, param: P) -> Self
        where S: Into<L>,
              P: Into<OptPolicy<T>>,
              T: Clone {

        let policy = param.into();
        self.short(short, policy.clone()).long(long, policy.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FnConfig<F, P> {
    fun:     F,
    phantom: PhantomData<fn() -> P>,
}

impl<F, P> FnConfig<F, P> {
    pub fn new(fun: F) -> Self
        where F: Fn(Flag<&str>) -> Option<P>,
              P: Into<Presence> {

        FnConfig {
            fun,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> F {
        self.fun
    }
}

impl<F, P> Deref for FnConfig<F, P> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.fun
    }
}

impl<F, P> DerefMut for FnConfig<F, P> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.fun
    }
}

impl<F, P> Config for FnConfig<F, P>
    where F: Fn(Flag<&str>) -> Option<P>,
          P: Into<Presence> {

    type Token = ();
    
    fn get_short_policy(&self, short: char) -> Option<OptPolicy<()>> {
        (self.fun)(Flag::Short(short)).map(Into::<OptPolicy<()>>::into)
    }

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<()>> {
        (self.fun)(Flag::Long(long)).map(Into::<OptPolicy<()>>::into)
    }
}

impl<T, U> Config for (T, U)
    where T: Config,
          U: Config<Token = T::Token> {

    type Token = T::Token;

    fn get_short_policy(&self, short: char) -> Option<OptPolicy<Self::Token>> {
        self.0.get_short_policy(short).or_else(||
            self.1.get_short_policy(short))
    }

    fn get_long_policy(&self, long: &str) -> Option<OptPolicy<Self::Token>> {
        self.0.get_long_policy(long).or_else(||
            self.1.get_long_policy(long))
    }
}

impl<L, P> Config for [(Flag<L>, P)]
    where L: Borrow<str>,
          P: Clone + Into<Presence> {

    type Token = ();

    fn get_short_policy(&self, name: char) -> Option<OptPolicy<()>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|pair| Into::<OptPolicy<()>>::into(pair.1.clone()))
    }

    fn get_long_policy(&self, name: &str) -> Option<OptPolicy<()>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|pair| Into::<OptPolicy<()>>::into(pair.1.clone()))
    }
}

impl<L, P, T> Config for [(Flag<L>, P, T)]
    where L: Borrow<str>,
          P: Clone + Into<Presence>,
          T: Clone {

    type Token = T;

    fn get_short_policy(&self, name: char) -> Option<OptPolicy<T>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|&(_, ref presence, ref token)|
                OptPolicy::new(presence.clone(), token.clone()))
    }

    fn get_long_policy(&self, name: &str) -> Option<OptPolicy<T>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|&(_, ref presence, ref token)|
                OptPolicy::new(presence.clone(), token.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Flag::*;
    use super::super::Presence::*;

    fn pres<T>(res: Option<OptPolicy<T>>) -> Option<Presence> {
        res.map(|policy| policy.presence)
    }

    fn check_config<C: Config>(config: C) {
        assert_eq!(pres(config.get_short_policy('a')), Some(Never) );
        assert_eq!(pres(config.get_short_policy('b')), None );
        assert_eq!(pres(config.get_short_policy('m')), Some(Always) );
        assert_eq!(pres(config.get_short_policy('r')), Some(IfAttached) );
        assert_eq!(pres(config.get_short_policy('s')), Some(Always) );
        assert_eq!(pres(config.get_short_policy('v')), Some(Never) );
        assert_eq!(pres(config.get_short_policy('q')), Some(Never) );

        assert_eq!(pres(config.get_long_policy("all")), Some(Never) );
        assert_eq!(pres(config.get_long_policy("bare")), None );
        assert_eq!(pres(config.get_long_policy("log")), Some(IfAttached) );
        assert_eq!(pres(config.get_long_policy("message")), Some(Always) );
        assert_eq!(pres(config.get_long_policy("rebase")), Some(IfAttached) );
        assert_eq!(pres(config.get_long_policy("strategy")), Some(Always) );
        assert_eq!(pres(config.get_long_policy("verbose")), Some(Never) );
        assert_eq!(pres(config.get_long_policy("quiet")), Some(Never) );
    }

    #[test]
    fn hash_config() {
        let config: HashConfig<String> = HashConfig::new()
            .both('a', "all",       false)
            .long(     "log",       IfAttached)
            .both('m', "message",   true)
            .both('r', "rebase",    IfAttached)
            .both('s', "strategy",  Always)
            .both('v', "verbose",   Never)
            .both('q', "quiet",     Never);

        check_config(config);
    }

    #[test]
    fn bare_bool_config() {
        let config = [
            (Short('a'), false),    (Long("all"), false),
            (Short('m'), true),     (Long("message"), true),
        ];

        assert_eq!( pres(config.get_short_policy('a')),        Some(Never) );
        assert_eq!( pres(config.get_short_policy('b')),        None );
        assert_eq!( pres(config.get_short_policy('m')),        Some(Always) );

        assert_eq!( pres(config.get_long_policy("all")),       Some(Never) );
        assert_eq!( pres(config.get_long_policy("bare")),      None );
        assert_eq!( pres(config.get_long_policy("message")),   Some(Always) );
    }

    #[test]
    fn bare_presence_config() {
        let config = [
            (Short('a'), Never),      (Long("all"),      Never),
                                      (Long("log"),      IfAttached),
            (Short('m'), Always),     (Long("message"),  Always),
            (Short('r'), IfAttached), (Long("rebase"),   IfAttached),
            (Short('s'), Always),     (Long("strategy"), Always),
            (Short('v'), Never),      (Long("verbose"),  Never),
            (Short('q'), Never),      (Long("quiet"),    Never),
        ];

        check_config(&config as &[_]);
    }

    #[test]
    fn pair_config() {
        let common_config = [
            (Short('v'), Never),      (Long("verbose"),  Never),
            (Short('q'), Never),      (Long("quiet"),    Never),
        ];

        let specific_config: HashConfig<&str> = HashConfig::new()
            .both('a', "all",       false)
            .long(     "log",       IfAttached)
            .both('m', "message",   true)
            .both('r', "rebase",    IfAttached)
            .both('s', "strategy",  Always);

        let config = (specific_config, &common_config as &[_]);

        check_config(config);
    }

    #[test]
    fn fn_config() {
        fn get(flag: Flag<&str>) -> Option<Presence> {
            let presence = match flag {
                Short('a')       => Never,
                Long("all")      => Never,
                Long("log")      => IfAttached,
                Short('m')       => Always,
                Long("message")  => Always,
                Short('r')       => IfAttached,
                Long("rebase")   => IfAttached,
                Short('s')       => Always,
                Long("strategy") => Always,
                Short('v')       => Never,
                Long("verbose")  => Never,
                Short('q')       => Never,
                Long("quiet")    => Never,
                _                => return None,
            };
            Some(presence)
        }

        let config = FnConfig::new(get);

        check_config(config);
        assert_eq!( config(Short('m')), Some(Always) );
    }

    #[test]
    fn allow_everything() {
        let config = FnConfig::new(|_| Some(IfAttached));
        assert_eq!( pres(config.get_short_policy('q')), Some(IfAttached) );
        assert_eq!( pres(config.get_long_policy("tralala")), Some(IfAttached) );
    }
}
