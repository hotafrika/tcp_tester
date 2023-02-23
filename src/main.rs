use tcp_tester;

fn main() {
    let args = tcp_tester::Args::parse_args();
    tcp_tester::run(args);
}
