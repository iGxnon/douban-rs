use std::path::Path;

pub trait Config {
    fn parse_all<P: AsRef<Path>>(ns_root: P) -> Vec<P>;
    fn parse_ignore<P: AsRef<Path>, F: FnOnce(P) -> bool>(ns_root: P, f: F) -> Vec<P>;
}
