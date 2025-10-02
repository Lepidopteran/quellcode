fn main() {
    dotenvy::dotenv().ok();
    color_eyre::install().ok();

    quellcode_lib::run();
}
