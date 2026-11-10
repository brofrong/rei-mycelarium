//! A panic is appended to the panic log (`MYCO_PANIC_LOG` here) with its message and location.

#[test]
fn panic_goes_to_the_log_file() {
    let path = std::env::temp_dir().join(format!("myco-panic-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    // SAFETY: this test binary has this one test, so no other thread reads the environment
    unsafe { std::env::set_var("MYCO_PANIC_LOG", &path) };
    myco_app::crash_log::install();
    let r = std::thread::spawn(|| panic!("boom at the crash log test")).join();
    assert!(r.is_err());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("=== panic at unix"), "{text}");
    assert!(text.contains("boom at the crash log test"), "{text}");
    assert!(text.contains("crash_log.rs:"), "{text}");
    let _ = std::fs::remove_file(&path);
}
