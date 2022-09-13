use std::{fmt, cmp::Ordering, str::FromStr};

use nom::{bytes::complete::take_while, character::complete::{alpha1, digit1}, IResult};

fn take_alpha(i: &str) -> (&str, &str) {
    let res: IResult<&str, _> = alpha1(i);
    
    res.unwrap_or((i, ""))
}

fn take_digits(i: &str) -> (&str, &str) {
    let res: IResult<&str, _> = digit1(i);

    res.unwrap_or((i, ""))
}

fn take_noalnum(i: &str) -> (&str, &str) {
    let res: IResult<&str, &str> = take_while(|c: char| !c.is_ascii_alphanumeric())(i);

    res.unwrap_or((i, ""))
}

const fn atend(a: &str, b: &str) -> bool {
    a.is_empty() || b.is_empty()
}

fn vercomp(a: &str, b: &str) -> Ordering {
    use Ordering::*;

    if a == b {
        return Equal;
    }

    let mut beg1 = a;
    let mut beg2 = b;

    let (rem1, rem2) = loop {
        // this catches those cases where one of the strings was empty to begin with
        if atend(beg1, beg2) {
            break (beg1, beg2);
        }

        let (rem1, sym1) = take_noalnum(beg1);
        let (rem2, sym2) = take_noalnum(beg2);

        if atend(rem1, rem2) {
            break (rem1, rem2);
        }

        let (sk1, sk2) = (sym1.len(), sym2.len());
        if sk1 != sk2 {
            return sk1.cmp(&sk2);
        }

        // next segment

        let is_num = rem1.starts_with(|c: char| c.is_ascii_digit());
        let take_fn = if is_num {
            take_digits
        } else {
            take_alpha
        };

        let (rem1, chk1) = take_fn(rem1);
        let (rem2, chk2) = take_fn(rem2);

        if chk2.is_empty() {
            // rem2 was not the same type as rem1
            // rpm arbitrarily assumes numeric segments are
            // greater than alphabetic ones

            return match is_num {
                true => Greater,
                false => Less,
            };
        }

        if is_num {
            // convert to u128, can't fail
            let n1 = u128::from_str(chk1).unwrap();
            let n2 = u128::from_str(chk2).unwrap();
            
            let cmp = n1.cmp(&n2);
            
            match cmp {
                Equal => {}, // continue
                v => return v,
            }
        } else {
            let cmp = chk1.cmp(&chk2);

            match cmp {
                Equal => {}, // continue
                v => return v,
            }
        }

        beg1 = rem1;
        beg2 = rem2;
    };
    
    if rem1.is_empty() && rem2.is_empty() {
        return Equal;
    }
    
    let alpha = |c: char| c.is_ascii_alphabetic();
    
    // strange RPM logic: 
    // 1. if one is empty and two is !alpha, two is newer;
    // 2. if one is alpha, two is newer;
    // 3. otherwise, one is newer.
    if rem1.is_empty() && !rem2.starts_with(alpha) || rem1.starts_with(alpha) {
        Less
    } else {
        Greater
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Version(String);

impl Version {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_components(&self) -> VersionComponents {
        let Version(evr) = self;

        // take the epoch if any
        let (version, epoch) = take_digits(evr);

        // remove any leading colon
        let some_nocol = version.strip_prefix(':');

        // find if there's an epoch
        let (epoch, version) = if !epoch.is_empty() && some_nocol.is_some() {
            (epoch, some_nocol.unwrap())
        } else {
            ("0", some_nocol.unwrap_or(evr)) 
        };

        // find the version terminator
        let (version, release) = match version.rsplit_once('-') {
            Some((version, release)) => (version, Some(release)),
            None => (version, None),
        };

        VersionComponents { epoch, version, release }
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Version {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl <'a> From<VersionComponents<'a>> for Version {
    fn from(vc: VersionComponents<'a>) -> Self {
        vc.to_version()
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self::new(s.to_owned())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_components().cmp(&other.as_components())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq)]
pub struct VersionComponents<'a> {
    pub epoch: &'a str,
    pub version: &'a str,
    pub release: Option<&'a str>,
}

impl <'a> VersionComponents<'a> {
    fn to_version(&self) -> Version {
        let Self { epoch, version, release } = *self;
        let release = release.unwrap_or("1");

        if epoch == "0" {
            format!("{}-{}", version, release)
        } else {
            format!("{}:{}-{}", epoch, version, release)
        }.into()
    }
}

impl <'a> Ord for VersionComponents<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        let res = vercomp(self.epoch, other.epoch)
            .then_with(|| vercomp(self.version, other.version));

        match (res, self.release, other.release) {
            (Equal, Some(rel1), Some(rel2)) => vercomp(rel1, rel2),
            (res, ..) => res,
        }
    }
}

impl <'a> PartialEq for VersionComponents<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl <'a> PartialOrd for VersionComponents<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}