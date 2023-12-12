fn main() {
    // let clang = match std::env::var_os("CLANG") {
    //     Some(val) => val,
    //     None => "clang-16".into(),
    // };

    cc::Build::new()
        .file("src/atomics.c")
        .static_flag(true)
        // .compiler(clang)
        .no_default_flags(true)
        // .flag("--target=riscv64")
        .flag("-march=rv64imc")
        .flag("-O3")
        .flag("-fvisibility=hidden")
        .flag("-fdata-sections")
        .flag("-ffunction-sections")
        .flag("-Wall")
        .flag("-Werror")
        .flag("-Wno-unused-parameter")
        .define("__SHARED_LIBRARY__", None)
        .compile("atomics-polyfill");
}
