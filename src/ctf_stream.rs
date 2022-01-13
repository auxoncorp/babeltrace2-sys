use crate::common_pipeline::CommonPipeline;
use crate::{
    BtResult, CtfPluginSourceLttnLiveInitParams, LoggingLevel, OwnedEvent, RunStatus,
    StreamProperties, TraceProperties,
};
use std::collections::{BTreeSet, VecDeque};

pub struct CtfStream {
    pipeline: CommonPipeline,
    metadata_recvd: bool,
}

impl CtfStream {
    pub fn new(
        log_level: LoggingLevel,
        params: &CtfPluginSourceLttnLiveInitParams,
    ) -> BtResult<Self> {
        let pipeline = CommonPipeline::new(log_level, params)?;
        Ok(CtfStream {
            pipeline,
            metadata_recvd: false,
        })
    }

    pub fn update(&mut self) -> BtResult<RunStatus> {
        let run_status = self.pipeline.graph.run_once()?;
        match run_status {
            RunStatus::Ok => {
                // The first time the pipeline indicates we have data means
                // we did the live session handshake and got the metadata
                self.metadata_recvd = true;
            }
            RunStatus::TryAgain => (),
            RunStatus::End => {
                log::debug!(
                    "CTF stream reached the end indicating the remote tracing session was closed"
                );
            }
        }
        Ok(run_status)
    }

    pub fn has_metadata(&self) -> bool {
        self.metadata_recvd
    }

    pub fn trace_properties(&self) -> &TraceProperties {
        &self.pipeline.proxy_state.as_ref().trace_properties
    }

    pub fn stream_properties(&self) -> &BTreeSet<StreamProperties> {
        &self.pipeline.proxy_state.as_ref().stream_properties
    }

    pub fn events_mut(&mut self) -> &mut VecDeque<OwnedEvent> {
        &mut self.pipeline.proxy_state.as_mut().events
    }

    pub fn events_chunk(&mut self) -> CtfStreamChunkIterator {
        CtfStreamChunkIterator { stream: self }
    }
}

pub struct CtfStreamChunkIterator<'a> {
    stream: &'a mut CtfStream,
}

impl<'a> Iterator for CtfStreamChunkIterator<'a> {
    type Item = OwnedEvent;

    fn next(&mut self) -> Option<Self::Item> {
        // Drain the previous message iterators batch of events
        self.stream.events_mut().pop_front()
    }
}
