use futures::Stream;
use rig::message::Message;
use rig::completion::CompletionModel;

pub type DynStream<M> = std::pin::Pin<
    Box<
        dyn Stream<
                Item = std::result::Result<
                    rig::agent::MultiTurnStreamItem<
                        <M as CompletionModel>::StreamingResponse
                    >,
                    rig::agent::StreamingError
                >
            > + Send
    >
>;

#[derive(Debug)]
pub struct UserInterruptionError {
    pub message: Message
}

impl std::fmt::Display for UserInterruptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation interrupted by user")
    }
}

impl std::error::Error for UserInterruptionError {}
