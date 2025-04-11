use std::collections::HashMap;

use anyhow::Result;
use axum::{body::Body, response::Response};
use dino_macros::{FromJs, IntoJs};
use rquickjs::{Context, Function, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
pub struct JsWorker {
    rt: Runtime,
    ctx: Context,
}

fn print(msg: String) {
    println!("{}", msg);
}

impl JsWorker {
    pub fn try_new(module: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        ctx.with(|ctx| {
            let global = ctx.globals();
            let ret: Object = ctx.eval(module)?;
            global.set("handlers", ret)?;
            let fun = Function::new(ctx.clone(), print)?.with_name("print")?;
            global.set("print", fun)?;

            Ok::<_, anyhow::Error>(())
        })?;
        Ok(Self { rt, ctx })
    }

    pub fn run(&self, name: &str, req: Req) -> anyhow::Result<Res> {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers: Object = global.get("handlers")?;
            let fun: Function = handlers.get(name)?;
            let v: Promise = fun.call((req,))?;

            Ok::<_, anyhow::Error>(v.finish()?)
        })
    }
}

#[derive(Debug, TypedBuilder, IntoJs)]
pub struct Req {
    #[builder(setter(into))]
    pub method: String,

    #[builder(setter(into))]
    pub url: String,

    #[builder(default)]
    pub query: HashMap<String, String>,

    #[builder(default)]
    pub params: HashMap<String, String>,

    #[builder(default)]
    pub headers: HashMap<String, String>,

    #[builder(default)]
    pub body: Option<String>,
}

// impl<'js> IntoJs<'js> for Req {
//     fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
//         let obj = Object::new(ctx.clone())?;
//         obj.set("headers", self.headers)?;
//         obj.set("method", self.method)?;
//         obj.set("url", self.url)?;
//         obj.set("body", self.body)?;

//         Ok(obj.into())
//     }
// }

#[derive(Debug, FromJs)]
pub struct Res {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl From<Res> for Response {
    fn from(value: Res) -> Self {
        let mut builder = Response::builder().status(value.status);
        for (k, v) in value.headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = value.body {
            builder.body(body.into()).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }
}

// impl<'js> FromJs<'js> for Res {
//     fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
//         let obj = value.into_object().unwrap();

//         let status: u16 = obj.get("status")?;
//         let headers: HashMap<String, String> = obj.get("headers")?;
//         let body: Option<String> = obj.get("body")?;

//         Ok(Res {
//             status,
//             headers,
//             body,
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_worker_should_run() {
        let code = r#"
            (function(){
                async function hello(req){
                    return {
                        status: 200,
                        headers: {
                            "content-type":"application.json"
                            },
                            body: JSON.stringify(req),
                        };
                    }
                    return{
                        hello:hello
                    };
            })();
        "#;
        let req = Req::builder()
            .method("GET")
            .url("https://example.com")
            .headers(HashMap::new())
            .build();

        let worker = JsWorker::try_new(code).unwrap();
        let ret = worker.run("hello", req).unwrap();
        assert_eq!(ret.status, 200);
    }
}
