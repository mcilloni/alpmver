use alpmver::Version;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// First argument to compare
    #[clap(value_parser)]
    v1: String,

    /// Second argument to compare
    #[clap(value_parser)]
    v2: String,
}

fn main() {
    let args = Args::parse();

    let v1 = Version::from(args.v1);
    let v2 = Version::from(args.v2);

    println!("{:?}", v1.cmp(&v2));    
}
