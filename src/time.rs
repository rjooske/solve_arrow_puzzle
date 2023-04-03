#[macro_export]
macro_rules! time {
    ($name:literal, $x:expr) => {{
        let before = std::time::Instant::now();
        let output = $x;
        print!($name);
        println!(" {:?}", before.elapsed());
        output
    }};
}

pub use time;
