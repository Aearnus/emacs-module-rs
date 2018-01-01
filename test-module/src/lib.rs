extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate emacs_module_bindings as emacs;

#[macro_use]
mod macros;

use emacs::{EmacsVal, EmacsRT, EmacsEnv};
use emacs::{Env, ToEmacs, Result};
use emacs::HandleFunc;
use std::os::raw;
use std::ptr;

/// This states that the module is GPL-compliant.
/// Emacs won't load the module if this symbol is undefined.
#[no_mangle]
#[allow(non_upper_case_globals)]
pub static plugin_is_GPL_compatible: libc::c_int = 0;

const MODULE: &str = "test-module";
lazy_static! {
    static ref MODULE_PREFIX: String = format!("{}/", MODULE);
}

fn test(env: &Env, _args: &[EmacsVal], _data: *mut raw::c_void) -> Result<EmacsVal> {
    env.to_emacs(5)?;
    match "1\0a".to_emacs(env) {
        Ok(_) => {
            println!("ok");
            env.call("message", &mut [
                "Should not get to this because we used a string with a zero byte".to_emacs(env)?
            ])?;
        },
        Err(_) => {
            println!("err");
            env.call("message", &mut [
                "Caught error here and continue".to_emacs(env)?
            ])?;
        }
    };

    println!("Start");
    let range = std::ops::Range { start: 0, end: 2usize.pow(22) };
    for i in range {
        println!("{}", i);
        let result = env.call("/", &mut[
            1.to_emacs(env)?,
            0.to_emacs(env)?,
        ]);
        match result {
            _ => continue
        }
    }
    println!("Stop");

    env.call("message", &mut [
        "(+ 1) -> %s".to_emacs(env)?,
        env.call("+", &mut [
            1.to_emacs(env)?
        ])?
    ])?;

    // Wrong type argument: symbolp, (throw-)
    env.call("throw-", &mut [
//        "How about this?".to_emacs(env)?
        1.to_emacs(env)?
    ])?;

    env.call("error", &mut [
//        "How about this?".to_emacs(env)?
        1.to_emacs(env)?
    ])?;
    env.call("+", &mut [
        "1\0".to_emacs(env)?,
        2.to_emacs(env)?,
    ])?;
    env.call("message", &mut [
        "Should not ever get here".to_emacs(env)?
    ])
}

emacs_subrs! {
    test -> f_test;
}

fn init(env: &Env) -> Result<EmacsVal> {
    make_prefix!(prefix, *MODULE_PREFIX);

    env.message("Hello, Emacs!")?;

    env.register(
        prefix!(test), f_test, 0..0,
        "", ptr::null_mut()
    )?;

    defuns! {
        env, *MODULE_PREFIX;

        inc, "1+", (env, x) {
            let i: i64 = env.from_emacs(x)?;
            (i + 1).to_emacs(env)
        }

        identity, "not even doing any conversion", (_env, x) {
            Ok(x)
        }

        "to-uppercase", "", (env, s) {
            let s: String = env.from_emacs(s)?;
            s.to_uppercase().to_emacs(env)
        }

        "calling-error", "", (env) {
            env.call("/", &mut [
                1.to_emacs(env)?,
                0.to_emacs(env)?,
            ])
        }

        "make-dec", "", (env) {
            fn dec(env: &Env, args: &[EmacsVal], _data: *mut raw::c_void) -> Result<EmacsVal> {
                let i: i64 = env.from_emacs(args[0])?;
                (i - 1).to_emacs(env)
            }
            emacs_subrs! {
                dec -> f_dec;
            }
            env.make_function(f_dec, 1..1, "decrement", ptr::null_mut())
        }
    }

    env.provide(MODULE)
}

/// Entry point for live-reloading during development.
#[no_mangle]
pub extern "C" fn emacs_rs_module_init(raw: *mut EmacsEnv) -> libc::c_int {
    match init(&Env::from(raw)) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

/// Entry point for Emacs' loader, for "production".
#[no_mangle]
pub extern "C" fn emacs_module_init(ert: *mut EmacsRT) -> libc::c_int {
    emacs_rs_module_init(Env::from(ert).raw())
}