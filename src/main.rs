use anyhow::Context;
use deno_core::anyhow::Error;
use deno_core::*;
use redis::Commands;
use std::rc::Rc;

#[op2(fast)]
fn op_cloudstate_object_set(
    state: &mut OpState,
    #[string] namespace: String,
    #[string] id: String,
    #[string] value: String,
) -> Result<(), Error> {
    let connection: &mut redis::Connection = state
        .try_borrow_mut::<redis::Connection>()
        .expect("Redis connection should be in OpState.");

    let key = format!("objects:{}:{}", namespace, id).to_string();
    connection.set(key, value)?;

    Ok(())
}

#[op2]
#[string]
fn op_cloudstate_object_get(
    state: &mut OpState,
    #[string] namespace: String,
    #[string] id: String,
) -> Result<Option<String>, Error> {
    let connection = state
        .try_borrow_mut::<redis::Connection>()
        .expect("Redis connection should be in OpState.");
    let key: &String = &format!("objects:{}:{}", namespace, id).to_string();

    let result = connection.get::<String, String>(key.to_string())?;

    Ok(Some(result))
}

#[op2(fast)]
fn op_cloudstate_object_root_set(
    state: &mut OpState,
    #[string] namespace: String,
    #[string] alias: String,
    #[string] id: String,
) -> Result<(), Error> {
    let connection = state
        .try_borrow_mut::<redis::Connection>()
        .expect("Redis connection should be in OpState.");

    let key = format!("roots:{}:{}", namespace, alias).to_string();
    connection.set(key, id)?;

    Ok(())
}

#[op2]
#[string]
fn op_cloudstate_object_root_get(
    state: &mut OpState,
    #[string] namespace: String,
    #[string] alias: String,
) -> Result<Option<String>, Error> {
    let connection = state
        .try_borrow_mut::<redis::Connection>()
        .expect("Redis connection should be in OpState.");

    let key: &String = &format!("roots:{}:{}", namespace, alias).to_string();

    let result = connection.get::<String, String>(key.to_string())?;

    Ok(Some(result))
}

deno_core::extension!(
  cloudstate,
  ops = [
    op_cloudstate_object_set,
    op_cloudstate_object_get,
    op_cloudstate_object_root_set,
    op_cloudstate_object_root_get,
  ],
  esm_entry_point = "ext:cloudstate/cloudstate.js",
  esm = [ dir "src", "cloudstate.js" ],
  state = | state: &mut OpState| {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Redis should be running.");
    let connection = client.get_connection().expect("Redis connection should be available.");
    state.put(connection);
  },
);

deno_core::extension!(
    superjson,
    esm_entry_point = "ext:superjson/superjson.js",
    esm = [ dir "src", "superjson.js" ],
);

deno_core::extension!(
    bootstrap,
    esm_entry_point = "ext:bootstrap/bootstrap.js",
    esm = [ dir "src", "bootstrap.js" ],
);

fn main() -> Result<(), Error> {
    let module_name = "test.js";
    let module_code = "
    const cloudstate = new Cloudstate('test');
    const object = { name: 'hello world' };
    cloudstate.setObject(object);
    cloudstate.setRoot(object, 'test');

    // const object = cloudstate.getRoot('test');
    // object.name = 'new world';
    // cloudstate.setObject(object);
  "
    .to_string();

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(FsModuleLoader)),
        extensions: vec![
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_console::deno_console::init_ops_and_esm(),
            bootstrap::init_ops_and_esm(),
            cloudstate::init_ops_and_esm(),
            superjson::init_ops_and_esm(),
        ],
        ..Default::default()
    });

    let main_module = resolve_path(
        module_name,
        &std::env::current_dir().context("Unable to get current working directory")?,
    )?;

    let future = async move {
        let mod_id = js_runtime
            .load_main_es_module_from_code(&main_module, module_code)
            .await?;

        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(Default::default()).await?;
        result.await
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
