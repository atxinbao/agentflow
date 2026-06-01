mod args;
mod legacy;
mod print;

fn main() -> anyhow::Result<()> {
    legacy::run()
}
