use crate::pipeline::DecoderPipeline;
use crate::{
    BtResult, CtfPluginSourceFsInitParams, LoggingLevel, OwnedEvent, RunStatus, StreamProperties,
    TraceProperties,
};
use std::collections::{BTreeSet, VecDeque};

pub struct CtfIterator {
    pipeline: DecoderPipeline,
    last_run_status: RunStatus,
}

impl CtfIterator {
    pub fn new(log_level: LoggingLevel, params: &CtfPluginSourceFsInitParams) -> BtResult<Self> {
        let mut pipeline = DecoderPipeline::new(log_level, params)?;

        // Do an initial run of the graph to connect and initialize all the components.
        // We'll have trace/stream metadata properties loaded and possibly some
        // events afterwards
        let last_run_status = pipeline.graph.run_once()?;

        Ok(CtfIterator {
            pipeline,
            last_run_status,
        })
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
}

impl Iterator for CtfIterator {
    type Item = BtResult<OwnedEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        // Drain the previous message iterators batch of events
        if let Some(event) = self.pipeline.proxy_state.as_mut().events.pop_front() {
            Some(Ok(event))
        } else {
            // Get another batch from upstream source component if not done
            match self.last_run_status {
                RunStatus::Ok | RunStatus::TryAgain => match self.pipeline.graph.run_once() {
                    Ok(last_run_status) => {
                        self.last_run_status = last_run_status;
                        self.events_mut().pop_front().map(Ok)
                    }
                    Err(e) => Some(Err(e)),
                },
                RunStatus::End => None,
            }
        }
    }
}
