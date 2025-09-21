fn main() {
    dotenvy::dotenv().ok();
    color_eyre::install().ok();

    quellcode::run();
}
