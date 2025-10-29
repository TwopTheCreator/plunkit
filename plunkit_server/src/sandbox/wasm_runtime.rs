use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::*;
use bytes::Bytes;

pub struct WasmRuntime {
    engine: Engine,
    store: Arc<Mutex<Store<RuntimeContext>>>,
    instance: Arc<Mutex<Option<Instance>>>,
}

pub struct RuntimeContext {
    pub world_id: String,
    pub memory_limit: usize,
    pub cpu_limit: u64,
}

impl RuntimeContext {
    pub fn new(world_id: String) -> Self {
        Self {
            world_id,
            memory_limit: 512 * 1024 * 1024,
            cpu_limit: 1_000_000_000,
        }
    }
}

impl WasmRuntime {
    pub async fn new(world_id: String) -> Result<Self> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);
        config.max_wasm_stack(2 * 1024 * 1024);
        config.cranelift_opt_level(OptLevel::Speed);

        let engine = Engine::new(&config)?;

        let context = RuntimeContext::new(world_id);
        let mut store = Store::new(&engine, context);

        store.limiter(|ctx| ctx as &mut dyn ResourceLimiter);
        store.set_fuel(1_000_000_000)?;

        Ok(Self {
            engine,
            store: Arc::new(Mutex::new(store)),
            instance: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn load_module(&self, wasm_bytes: &[u8]) -> Result<()> {
        let module = Module::new(&self.engine, wasm_bytes)
            .context("Failed to compile WASM module")?;

        let mut store = self.store.lock().await;

        let mut linker = Linker::new(&self.engine);

        self.add_host_functions(&mut linker)?;

        let instance = linker
            .instantiate_async(&mut *store, &module)
            .await
            .context("Failed to instantiate WASM module")?;

        let mut instance_lock = self.instance.lock().await;
        *instance_lock = Some(instance);

        Ok(())
    }

    fn add_host_functions(&self, linker: &mut Linker<RuntimeContext>) -> Result<()> {
        linker.func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, RuntimeContext>, ptr: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return,
                };

                let data = mem
                    .data(&caller)
                    .get(ptr as usize..(ptr + len) as usize)
                    .unwrap_or(&[]);

                if let Ok(s) = std::str::from_utf8(data) {
                    tracing::info!("WASM Log: {}", s);
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "get_block",
            |mut _caller: Caller<'_, RuntimeContext>, x: i32, y: i32, z: i32| -> i32 {
                0
            },
        )?;

        linker.func_wrap(
            "env",
            "set_block",
            |mut _caller: Caller<'_, RuntimeContext>, x: i32, y: i32, z: i32, block_id: i32| {
            },
        )?;

        linker.func_wrap(
            "env",
            "spawn_entity",
            |mut _caller: Caller<'_, RuntimeContext>,
             entity_type: i32,
             x: f64,
             y: f64,
             z: f64| -> i32 {
                0
            },
        )?;

        Ok(())
    }

    pub async fn call_function(&self, name: &str, args: &[Val]) -> Result<Vec<Val>> {
        let mut store = self.store.lock().await;
        let instance_lock = self.instance.lock().await;

        let instance = instance_lock
            .as_ref()
            .context("No WASM instance loaded")?;

        let func = instance
            .get_func(&mut *store, name)
            .context(format!("Function '{}' not found", name))?;

        let mut results = vec![Val::I32(0); func.ty(&*store).results().len()];

        func.call_async(&mut *store, args, &mut results)
            .await
            .context(format!("Failed to call function '{}'", name))?;

        Ok(results)
    }

    pub async fn tick(&self) -> Result<()> {
        self.call_function("tick", &[]).await?;
        Ok(())
    }

    pub async fn on_player_join(&self, player_id: i32) -> Result<()> {
        self.call_function("on_player_join", &[Val::I32(player_id)])
            .await?;
        Ok(())
    }

    pub async fn on_block_break(&self, x: i32, y: i32, z: i32, player_id: i32) -> Result<()> {
        self.call_function(
            "on_block_break",
            &[
                Val::I32(x),
                Val::I32(y),
                Val::I32(z),
                Val::I32(player_id),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn get_remaining_fuel(&self) -> Result<u64> {
        let store = self.store.lock().await;
        Ok(store.get_fuel().unwrap_or(0))
    }

    pub async fn refuel(&self, fuel: u64) -> Result<()> {
        let mut store = self.store.lock().await;
        store.set_fuel(fuel)?;
        Ok(())
    }
}

impl ResourceLimiter for RuntimeContext {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool, Error> {
        Ok(desired <= self.memory_limit)
    }

    fn table_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> Result<bool, Error> {
        Ok(desired <= 10000)
    }
}
