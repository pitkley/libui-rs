extern crate cmake;
use cmake::Config;

use std::env;
use std::path::Path;
use std::process::Command;

// If libui is statically built for Windows using MinGW, the Rust linker needs to know about the
// various libraries libui depends on to be able to successfully link the DLL in.
//
// The following list of libraries was taken from:
// https://github.com/andlabs/libui/blob/6a513038f43b0b189daf1152fe35010788651e71/windows/CMakeLists.txt#L85
const WINDOWS_GNU_LINKER_FLAGS: &[&str] = &[
    "comctl32",
    "gdi32",
    "uxtheme",
    "msimg32",
    "comdlg32",
    "d2d1",
    "dwrite",
    "ole32",
    "oleaut32",
    "oleacc",
    "uuid",
    "windowscodecs",
    // `stdc++` might not be explicitely needed if you use the right linker, adding it gives a
    // higher compatibility though.
    "stdc++",
];

fn main() {
    // Fetch the submodule if needed
    if cfg!(feature = "fetch") {
        // Init or update the submodule with libui if needed
        if !Path::new("libui/.git").exists() {
            Command::new("git")
                .args(&["version"])
                .status()
                .expect("Git does not appear to be installed. Error");
            Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status()
                .expect("Unable to init libui submodule. Error");
        } else {
            Command::new("git")
                .args(&["submodule", "update", "--recursive"])
                .status()
                .expect("Unable to update libui submodule. Error");
        }
    }

    // Deterimine if we're building for Windows with either MSVC or GNU (MinGW)
    let target = env::var("TARGET").unwrap();
    let windows_msvc = target.contains("msvc");
    let windows_gnu = target.contains("windows-gnu");

    // Build libui if needed. Otherwise, assume it's in lib/
    let mut dst;
    if cfg!(feature = "build") {
        let mut cmake = Config::new("libui");
        cmake.build_target("").profile("release");
        if windows_gnu {
            // libui does not yet support building a shared library for Windows using MinGW, thus
            // we need to build a static library instead.
            cmake.define("BUILD_SHARED_LIBS", "OFF");
        }
        dst = cmake.build();

        let mut postfix = Path::new("build").join("out");
        if windows_msvc {
            postfix = postfix.join("Release");
        }
        dst = dst.join(&postfix);
    } else {
        dst = env::current_dir()
            .expect("Unable to retrieve current directory location.");
        dst.push("lib");
    }

    let libname;
    if windows_msvc {
        libname = "libui";
    } else {
        libname = "ui";
    }

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib={}", libname);
    if windows_gnu {
        // As mentioned above, we need to specify a number of libraries that have to be linked in.
        print!("cargo:rustc-flags=");
        for linker_flag in WINDOWS_GNU_LINKER_FLAGS {
            print!("-l {} ", linker_flag);
        }
        println!();
    }
}
