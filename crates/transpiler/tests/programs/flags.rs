use std::collections::HashSet;
use std::fmt;
use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor, Not, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Flags(u8);

impl Flags {
    const NONE: Flags = Flags(0);
    const READ: Flags = Flags(1);
    const WRITE: Flags = Flags(2);
    const EXEC: Flags = Flags(4);
    const ALL: Flags = Flags(7);

    fn contains(self, other: Flags) -> bool {
        self.0 & other.0 == other.0
    }

    fn bits(self) -> u8 {
        self.0
    }

    fn names(self) -> Vec<&'static str> {
        [(Flags::READ, "read"), (Flags::WRITE, "write"), (Flags::EXEC, "exec")]
            .iter()
            .filter(|(f, _)| self.contains(*f))
            .map(|(_, n)| *n)
            .collect()
    }

    fn from_bits(bits: u8) -> Option<Flags> {
        if bits & !Flags::ALL.0 == 0 { Some(Flags(bits)) } else { None }
    }

    fn count(self) -> u32 {
        self.0.count_ones()
    }
}

impl BitOr for Flags {
    type Output = Flags;
    fn bitor(self, rhs: Flags) -> Flags {
        Flags(self.0 | rhs.0)
    }
}

impl BitAnd for Flags {
    type Output = Flags;
    fn bitand(self, rhs: Flags) -> Flags {
        Flags(self.0 & rhs.0)
    }
}

impl BitXor for Flags {
    type Output = Flags;
    fn bitxor(self, rhs: Flags) -> Flags {
        Flags(self.0 ^ rhs.0)
    }
}

impl Not for Flags {
    type Output = Flags;
    fn not(self) -> Flags {
        Flags(!self.0 & Flags::ALL.0)
    }
}

impl Sub for Flags {
    type Output = Flags;
    fn sub(self, rhs: Flags) -> Flags {
        Flags(self.0 & !rhs.0)
    }
}

impl BitOrAssign for Flags {
    fn bitor_assign(&mut self, rhs: Flags) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "-");
        }
        write!(f, "{}", self.names().join("|"))
    }
}

impl fmt::Binary for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:03b}", self.0)
    }
}

fn main() {
    let rw = Flags::READ | Flags::WRITE;
    let mut perms = rw;
    perms |= Flags::EXEC;
    println!("{rw} {perms} {} {} {:?}", !rw, perms - Flags::WRITE, rw & Flags::EXEC);
    println!("{} {} {} {}", rw.contains(Flags::READ), rw.contains(Flags::EXEC), perms == Flags::ALL, (rw ^ perms) == Flags::EXEC);
    println!("{:?} {:?} {:?}", Flags::from_bits(5).map(|f| f.to_string()), Flags::from_bits(9), Flags::default() == Flags::NONE);
    println!("{:b} {:b} {} {}", rw, Flags::NONE, perms.bits(), perms.count());
    let mut set: HashSet<Flags> = HashSet::new();
    for bits in 0..8u8 {
        set.insert(Flags(bits) & rw);
    }
    let mut members: Vec<Flags> = set.into_iter().collect();
    members.sort();
    println!("{}", members.iter().map(|f| format!("{f}")).collect::<Vec<_>>().join(", "));
    println!("{:?} {:?}", members.iter().max(), members.iter().map(|f| f.count()).sum::<u32>());
    let combos: Vec<String> = (0..8u8).filter_map(Flags::from_bits).filter(|f| f.contains(Flags::READ)).map(|f| format!("{f}={}", f.bits())).collect();
    println!("{:?}", combos);
    println!("{}", (Flags::READ | Flags::EXEC) < (Flags::WRITE | Flags::EXEC));
}
