fn main() -> std::process::ExitCode {
    let (status, output) = curriculum::projection::run(std::env::args().skip(1));
    println!("{output}");
    status
}
