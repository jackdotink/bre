use cmake::Config;

fn main() {
    let dst = Config::new("luau")
        .define("LUAU_BUILD_CLI", "OFF")
        .define("LUAU_BUILD_TESTS", "OFF")
        .define("LUAU_BUILD_WEB", "OFF")
        .define("LUAU_EXTERN_C", "ON")
        .define("LUAU_STATIC_CRT", "ON")
        .no_build_target(true)
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=Luau.VM");
    println!("cargo:rustc-link-lib=static=Luau.Compiler");
    println!("cargo:rustc-link-lib=static=Luau.Ast");
    println!("cargo:rustc-link-lib=stdc++");
}
