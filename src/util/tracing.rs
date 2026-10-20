use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;
use tracing_subscriber::util::TryInitError;
use tracing_subscriber::{
    fmt, fmt::format::FmtSpan, layer::Context, prelude::*, registry::LookupSpan, Layer,
};

/// #### Description
/// Initialize the default [tracing_subscriber].
pub fn init_default_tracing_subscriber(ansi_color: bool) -> Result<(), TryInitError> {
    let logger = fmt::layer()
        .with_ansi(ansi_color)
        .with_span_events(FmtSpan::ENTER /*| FmtSpan::CLOSE*/)
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(logger).try_init()
}

/// #### Description
/// Initialize a [tracing_subscriber] for runtime profiling.
pub fn init_profiler_tracing_subscriber() -> Result<RuntimeReporter, TryInitError> {
    let profiler = RuntimeProfiler::default();
    let reporter = profiler.reporter();
    tracing_subscriber::registry().with(profiler).try_init()?;
    Ok(reporter)
}

pub struct RuntimeReporter {
    times: Arc<Mutex<HashMap<&'static str, u128>>>,
}
impl RuntimeReporter {
    /// #### Description
    /// Print the total recorded times for all spans.
    pub fn report(&self) {
        let times = self.times.lock().unwrap();
        let mut times = times.iter().collect::<Vec<(_, _)>>();
        times.sort_by_key(|x| u128::MAX - x.1);
        println!("Runtime profiling report:");
        for (name, &time) in times.iter() {
            println!("  {:<30} {:>10.3} ms", name, (time as f64) / 1_000_000.0);
        }
    }
}

#[derive(Default)]
pub(crate) struct RuntimeProfiler {
    times: Arc<Mutex<HashMap<&'static str, u128>>>,
    stack: Arc<Mutex<VecDeque<Instant>>>,
}
impl RuntimeProfiler {
    fn reporter(&self) -> RuntimeReporter {
        RuntimeReporter { times: self.times.clone() }
    }
}
impl<S: Subscriber> Layer<S> for RuntimeProfiler
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        if ctx.span(id).is_some() {
            self.stack.lock().unwrap().push_back(Instant::now());
        }
    }

    fn on_exit(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut stack = self.stack.lock().unwrap();
            let start = stack.pop_back().unwrap();
            // only top level spans
            if stack.is_empty() {
                let elapsed = start.elapsed().as_nanos();
                let name = span.metadata().name();
                let mut times = self.times.lock().unwrap();
                let total = times.entry(name).or_insert(0);
                *total += elapsed;
            }
        }
    }
}
