#![deny(warnings, clippy::all)]

use std::fs;

fn main() {
    let mut config = autotools::Config::new("vendor/babeltrace");
    config.reconf("-vif");
    config.enable("built-in-plugins", None);
    config.enable("silent-rules", None);
    config.disable("debug-info", None);
    config.disable("man-pages", None);
    config.disable("glibtest", None);
    config.disable("doxygen-doc", None);
    config.disable("doxygen-html", None);
    config.disable("maintainer-mode", None);
    config.disable("dependency-tracking", None);
    config.disable_shared();
    config.enable_static();
    config.fast_build(true);

    if cfg!(debug_assertions) {
        config.enable("asan", None);
        config.env("BABELTRACE_DEV_MODE", "1");
        config.env("BABELTRACE_DEBUG_MODE", "1");
        config.env("BABELTRACE_MINIMAL_LOG_LEVEL", "INFO");
    } else {
        config.disable("asan", None);
    }

    let babeltrace_path = config.build();

    let glib2 = pkg_config::Config::new()
        .atleast_version("2.0.0")
        .statik(true)
        .probe("glib-2.0")
        .expect("Failed to find glib-2.0 pkg-config");

    let gmod2 = pkg_config::Config::new()
        .atleast_version("2.0.0")
        .statik(true)
        .probe("gmodule-2.0")
        .expect("Failed to find gmodule-2.0 pkg-config");

    let pcre = pkg_config::Config::new()
        .statik(true)
        .probe("libpcre")
        .expect("Failed to find libpcre pkg-config");

    if cfg!(feature = "test") {
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    }

    println!(
        "cargo:rustc-link-search=native={}/lib",
        babeltrace_path.display()
    );
    println!("cargo:rustc-link-lib=static=babeltrace2");
    println!("cargo:rustc-link-lib=static=babeltrace2-ctf-writer");

    let plugin_path = babeltrace_path.join("lib/babeltrace2/plugins");
    fs::copy(
        plugin_path.join("babeltrace-plugin-utils.a"),
        plugin_path.join("libbabeltrace-plugin-utils.a"),
    )
    .unwrap();
    fs::copy(
        plugin_path.join("babeltrace-plugin-ctf.a"),
        plugin_path.join("libbabeltrace-plugin-ctf.a"),
    )
    .unwrap();
    println!(
        "cargo:rustc-link-search=native={}/lib/babeltrace2/plugins/",
        babeltrace_path.display()
    );
    println!("cargo:rustc-link-lib=static=babeltrace-plugin-utils");
    println!("cargo:rustc-link-lib=static=babeltrace-plugin-ctf");

    println!(
        "cargo:rustc-link-search=native={}",
        gmod2.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=static={}", gmod2.libs[0]);
    println!(
        "cargo:rustc-link-search=native={}",
        glib2.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=static={}", glib2.libs[0]);
    println!(
        "cargo:rustc-link-search=native={}",
        pcre.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=static={}", pcre.libs[0]);

    println!("cargo:rustc-link-lib=dylib=c");
}
