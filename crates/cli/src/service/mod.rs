/// This service acts as the single entry point for:
/// - the GRPC server
/// - the main event loop
/// The server hands a message to the event loop to ingest events synchronously.
/// Meanwhile the event loop is always running and waiting for events.
