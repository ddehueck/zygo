mod paths;
mod stream_processor;

mod db;
mod service;

// This is the single entrypoint for the local Zygo service.
pub use service::ZygoLocalService;
