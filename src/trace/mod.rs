use std::{collections::HashMap, sync::Arc};

use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{
        Address, BlockId, BlockNumber, Bytes, CallConfig, GethDebugBuiltInTracerConfig,
        GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
        GethDebugTracingCallOptions, GethTrace::Known, GethTraceFrame::CallTracer, NameOrAddress,
        Transaction,
    },
};

fn flatten(frame: &ethers::types::CallFrame, flattened: &mut HashMap<Address, Bytes>) {
    match &frame.to {
        Some(a) => {
            if let NameOrAddress::Address(addr) = a {
                flattened.insert(*addr, frame.input.clone());
            }
        }
        None => {} // Ignore contract creations
    }
    if let Some(child_calls) = &frame.calls {
        for child in child_calls {
            flatten(child, flattened);
        }
    }
}

pub async fn get_flattened_trace(
    tx: Transaction,
    provider: Arc<Provider<Ws>>,
) -> Option<HashMap<Address, Bytes>> {
    let mut opts = GethDebugTracingCallOptions::default();
    opts.tracing_options.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
        GethDebugBuiltInTracerConfig::CallTracer(CallConfig {
            only_top_call: Some(false),
            with_log: Some(false),
        }),
    ));
    opts.tracing_options.timeout = Some("1s".to_string());
    opts.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(
        GethDebugBuiltInTracerType::CallTracer,
    ));
    let block_id = BlockId::Number(BlockNumber::Latest);
    let traces = provider.debug_trace_call(&tx, Some(block_id), opts).await;
    if let Ok(traces) = traces {
        // Recursively flatten the CallFrame
        // mapping of To -> Bytes
        let mut flattened: HashMap<Address, Bytes> = HashMap::new();
        if let Known(known_trace) = traces {
            if let CallTracer(t) = known_trace {
                flatten(&t, &mut flattened);
                return Some(flattened);
            }
        }
    }
    None
}
