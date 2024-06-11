use env_logger::Builder;
use libbitcoinkernel_sys::{
    ChainType, Context, ContextBuilder, KernelError,
    KernelNotificationInterfaceCallbackHolder, LogCallback, Logger,
};
use log::LevelFilter;

pub fn setup_logging() -> Result<Logger, KernelError> {
    let mut builder = Builder::from_default_env();
    builder.filter(None, LevelFilter::Info).init();

    let callback = |message: &str| {
        log::info!(
            target: "libbitcoinkernel", 
            "{}", message.strip_suffix("\r\n").or_else(|| message.strip_suffix('\n')).unwrap_or(message));
    };

    Logger::new(LogCallback::new(callback))
}

pub fn create_context(network: ChainType) -> Context {
    ContextBuilder::new()
        .chain_type(network)
        .unwrap()
        .kn_callbacks(Box::new(KernelNotificationInterfaceCallbackHolder {
            kn_block_tip: Box::new(|_state, _block_index| {}),
            kn_header_tip: Box::new(|_state, _height, _timestamp, _presync| {}),
            kn_progress: Box::new(|_title, _progress, _resume_possible| {}),
            kn_warning: Box::new(|_warning| {}),
            kn_flush_error: Box::new(|_message| {}),
            kn_fatal_error: Box::new(|_message| {}),
        }))
        .unwrap()
        .build()
        .unwrap()
}

