fn main() {
    let s = "legacyPackages.x86_64-linux.emacs30-nox.dtrn";

    let sp: Vec<&str> = s.splitn(3, ".").collect();
    println!("{:?}", sp.get(2).unwrap());
}
