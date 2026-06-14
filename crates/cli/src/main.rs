mod active;
mod args;
mod legacy;
mod print;
mod retirement;

fn main() -> anyhow::Result<()> {
    legacy::run()
}
