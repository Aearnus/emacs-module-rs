use emacs::{Env, CallEnv, Value, IntoLisp, Result};
use emacs::ErrorKind::{self, Signal, Throw};

use super::MODULE_PREFIX;

fn lisp_divide(env: &CallEnv) -> Result<i64> {
    let x = env.get_arg(0);
    let y = env.get_arg(1);

    fn inner(env: &Env, x: i64, y: i64) -> Result<Value> {
        call!(env, "/", x, y)
    }

    fn foo<'e>(env: &'e Env, x: Value, y: Value) -> Result<Value<'e>> {
        inner(
            env,
            x.into_rust()?,
            y.into_rust()?,
        )
    }

    foo(env, x, y)?.into_rust()
}

fn get_type(env: &CallEnv) -> Result<Value> {
    let f = env.get_arg(0);
    match env.call("funcall", &[f]) {
        Err(error) => {
            if let Some(&Signal { ref symbol, .. }) = error.downcast_ref::<ErrorKind>() {
                unsafe {
                    return Ok(symbol.value(env))
                }
            }
            Err(error)
        },
        v => v,
    }
}

fn catch(env: &CallEnv) -> Result<Value> {
    let expected_tag = env.get_arg(0);
    let f = env.get_arg(1);
    match env.call("funcall", &[f]) {
        Err(error) => {
            if let Some(&Throw { ref tag, ref value }) = error.downcast_ref::<ErrorKind>() {
                unsafe {
                    if env.eq(tag.value(env), expected_tag) {
                        return Ok(value.value(env))
                    }
                }
            }
            Err(error)
        },
        v => v,
    }
}

fn parse_arg(env: &CallEnv) -> Result<String> {
    let i: i64 = env.parse_arg(0)?;
    let s: String = env.parse_arg(i as usize)?;
    Ok(s)
}

pub fn init(env: &Env) -> Result<()> {
    emacs_export_functions! {
        env, format!("{}error:", *MODULE_PREFIX), {
            "lisp-divide" => (lisp_divide , 2..2),
            "get-type"    => (get_type    , 1..1),
            "catch"       => (catch       , 2..2),
            "parse-arg"   => (parse_arg   , 2..5),
        }
    }

    Ok(())
}
