const VERSION: usize = env!("VERSION", 1);

#[executable]
fn main() {
    assert(VERSION == 2, 'VERSION is not 2');
}
