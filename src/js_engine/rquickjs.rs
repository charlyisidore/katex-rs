//! JS Engine implemented by [rquickjs](https://crates.io/crates/rquickjs).

use std::collections::HashMap;

use rquickjs::IntoJs;

use crate::{
    error::{Error, Result},
    js_engine::{JsEngine, JsValue},
};

/// rquickjs Engine.
pub struct Engine {
    context: rquickjs::Context,
}

impl JsEngine for Engine {
    type JsValue<'a> = Value<'a>;

    fn new() -> Result<Self> {
        Ok(Self {
            context: rquickjs::Runtime::new()
                .and_then(|runtime| rquickjs::Context::full(&runtime))
                .map_err(|e| Error::JsInitError(format!("{e}")))?,
        })
    }

    fn eval<'a>(&'a self, code: &str) -> Result<Self::JsValue<'a>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: ctx
                    .eval(code)
                    .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
            })
        })
    }

    fn call_function<'a>(
        &'a self,
        func_name: &str,
        args: impl Iterator<Item = Self::JsValue<'a>>,
    ) -> Result<Self::JsValue<'a>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: ctx
                    .globals()
                    .get::<_, rquickjs::Function>(func_name)
                    .and_then(|function| {
                        let mut js_args = rquickjs::function::Args::new_unsized(ctx.clone());
                        js_args.push_args(args.map(|v| v.value))?;
                        function.call_arg(js_args)
                    })
                    .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
            })
        })
    }

    fn create_bool_value(&self, input: bool) -> Result<Self::JsValue<'_>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: rquickjs::Persistent::save(
                    &ctx,
                    input
                        .into_js(&ctx)
                        .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
                ),
            })
        })
    }

    fn create_int_value(&self, input: i32) -> Result<Self::JsValue<'_>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: rquickjs::Persistent::save(
                    &ctx,
                    input
                        .into_js(&ctx)
                        .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
                ),
            })
        })
    }

    fn create_float_value(&self, input: f64) -> Result<Self::JsValue<'_>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: rquickjs::Persistent::save(
                    &ctx,
                    input
                        .into_js(&ctx)
                        .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
                ),
            })
        })
    }

    fn create_string_value(&self, input: String) -> Result<Self::JsValue<'_>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: rquickjs::Persistent::save(
                    &ctx,
                    input
                        .into_js(&ctx)
                        .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
                ),
            })
        })
    }

    fn create_object_value<'a>(
        &'a self,
        input: impl Iterator<Item = (String, Self::JsValue<'a>)>,
    ) -> Result<Self::JsValue<'a>> {
        self.context.with(|ctx| {
            Ok(Value {
                context: &self.context,
                value: rquickjs::Persistent::save(
                    &ctx,
                    input
                        .map(|(k, v)| (k, v.value))
                        .collect::<HashMap<_, _>>()
                        .into_js(&ctx)
                        .map_err(|e| Error::JsExecError(error_to_string(&ctx, e)))?,
                ),
            })
        })
    }
}

/// rquickjs Value.
pub struct Value<'a> {
    value: rquickjs::Persistent<rquickjs::Value<'static>>,
    context: &'a rquickjs::Context,
}

impl<'a> std::fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Value").field("value", &self.value).finish()
    }
}

impl<'a> JsValue<'a> for Value<'a> {
    fn into_string(self) -> Result<String> {
        self.context.with(|ctx| {
            self.value
                .restore(&ctx)
                .map_err(|e| Error::JsValueError(error_to_string(&ctx, e)))?
                .into_string()
                .ok_or_else(|| Error::JsValueError("cannot convert value to string".to_owned()))?
                .to_string()
                .map_err(|e| Error::JsValueError(error_to_string(&ctx, e)))
        })
    }
}

fn error_to_string(ctx: &rquickjs::Ctx, e: rquickjs::Error) -> String {
    match e {
        rquickjs::Error::Exception => format!("{e}: {:?}", ctx.catch()),
        _ => format!("{e}"),
    }
}
