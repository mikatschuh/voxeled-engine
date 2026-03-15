#[macro_export]
macro_rules! print_info {
    () => {
        println!("[INFO]");
    };
    ($($arg:tt)*) => {
        println!("[INFO] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! print_warning {
    () => {
        println!("[WARNING]");
    };
    ($($arg:tt)*) => {
        println!("[WARNING] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! print_error {
    () => {
        println!("[ERROR]");
    };
    ($($arg:tt)*) => {
        println!("[ERROR] {}", format_args!($($arg)*));
    };
}
