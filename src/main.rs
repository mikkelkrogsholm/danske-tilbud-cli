use usage_rs::Cli;

/// Grundlag for et dansk dagligvare-CLI. Tilbudssøgning er endnu ikke implementeret.
#[derive(Cli)]
#[usage(bin = "tilbud", version = "0.1.0")]
struct Tilbud {}

fn main() {
    Tilbud::parse();
    println!("Tilbudssøgning er endnu ikke implementeret. Se --help.");
}
