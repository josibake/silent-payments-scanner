use env_logger::Builder;
use libbitcoinkernel_sys::{
    ChainType, Context, ContextBuilder, KernelError, KernelNotificationInterfaceCallbackHolder,
    Log, Logger,
};
use log::LevelFilter;

pub struct MainLog {}

impl Log for MainLog {
    fn log(&self, message: &str) {
        log::info!(
            target: "libbitcoinkernel", 
            "{}", message.strip_suffix("\r\n").or_else(|| message.strip_suffix('\n')).unwrap_or(message));
    }
}

pub fn setup_logging() -> Result<Logger<MainLog>, KernelError> {
    let mut builder = Builder::from_default_env();
    builder.filter(None, LevelFilter::Info).init();
    Logger::new(MainLog {})
}

pub fn create_context(network: ChainType) -> Context {
    ContextBuilder::new()
        .chain_type(network)
        .unwrap()
        .kn_callbacks(Box::new(KernelNotificationInterfaceCallbackHolder {
            kn_block_tip: Box::new(|_state, _block_index| {}),
            kn_header_tip: Box::new(|_state, _height, _timestamp, _presync| {}),
            kn_progress: Box::new(|_title, _progress, _resume_possible| {}),
            kn_warning_set: Box::new(|_warning, _message| {}),
            kn_warning_unset: Box::new(|_warning| {}),
            kn_flush_error: Box::new(|_message| {}),
            kn_fatal_error: Box::new(|_message| {}),
        }))
        .unwrap()
        .build()
        .unwrap()
}
