use crate::{
    BoxedRawProxyPluginState, BtResult, ComponentClassFilter, ComponentClassSink,
    ComponentClassSource, ComponentFilter, ComponentSink, ComponentSource, CtfPlugin,
    CtfPluginSourceInitParams, Error, Graph, Logger, LoggingLevel, OwnedEvent, ProxyPlugin,
    RunStatus, StreamProperties, TraceProperties, UtilsPlugin,
};
use std::collections::{BTreeSet, VecDeque};

pub struct TraceIterator {
    _utils_plugin: UtilsPlugin,
    _ctf_plugin: CtfPlugin,
    _proxy_plugin: ProxyPlugin,
    _ctf_src_class: ComponentClassSource,
    _muxer_filter_class: ComponentClassFilter,
    _proxy_sink_class: ComponentClassSink,
    _ctf_src: ComponentSource,
    _ctf_params: CtfPluginSourceInitParams,
    _muxer_filter: ComponentFilter,
    _proxy_sink: ComponentSink,
    graph: Graph,
    last_run_status: RunStatus,
    proxy_state: BoxedRawProxyPluginState,
}

impl TraceIterator {
    pub fn new(log_level: LoggingLevel, ctf_params: CtfPluginSourceInitParams) -> BtResult<Self> {
        Logger::set_level(log_level);

        // Load builtin plugins we need
        let utils_plugin = UtilsPlugin::load()?;
        let ctf_plugin = CtfPlugin::load()?;

        // Load our custom proxy plugin
        let proxy_plugin = ProxyPlugin::load()?;

        // Borrow the component classes from the plugins
        let ctf_src_class = ctf_plugin.borrow_fs_source_component_class()?;
        let muxer_filter_class = utils_plugin.borrow_muxer_filter_component_class()?;
        let proxy_sink_class = proxy_plugin.borrow_output_sink_component_class_by_name()?;

        let mut graph = Graph::new()?;

        // Add components to the processing graph
        let ctf_src = graph.add_source_component(
            &ctf_src_class,
            CtfPlugin::graph_node_name(),
            ctf_params.params(),
            log_level,
        )?;

        let muxer_filter = graph.add_filter_component(
            &muxer_filter_class,
            UtilsPlugin::graph_node_name(),
            log_level,
        )?;

        let mut proxy_state = BoxedRawProxyPluginState::new();
        let proxy_sink = graph.add_sink_component_with_initialize_method_data(
            &proxy_sink_class,
            ProxyPlugin::graph_node_name(),
            proxy_state.as_raw() as _,
            log_level,
        )?;

        // Connect all available source output ports to the muxer filter input ports
        let num_ctf_out_ports = ctf_src.get_output_port_count();
        if num_ctf_out_ports == 0 {
            log::debug!("Input path doesn't appear to contain any stream data");
            return Err(Error::CtfSourceMissingOutputPorts);
        }
        let num_proxy_in_ports = proxy_sink.get_input_port_count();
        if num_proxy_in_ports == 0 {
            return Err(Error::ProxySinkMissingInputPort);
        }
        log::debug!("Connecting {} CTF source ports to muxer", num_ctf_out_ports);
        for pidx in 0..num_ctf_out_ports {
            let in_port = muxer_filter.borrow_input_port_by_index(pidx)?;
            let out_port = ctf_src.borrow_output_port_by_index(pidx)?;
            graph.connect_ports(&out_port, &in_port)?;
        }

        // Connect the mux'd filter output port to the proxy sink input port
        log::debug!("Connecting muxer port to proxy sink");
        let in_port = proxy_sink.borrow_input_port_by_index(0)?;
        let out_port = muxer_filter.borrow_output_port_by_index(0)?;
        graph.connect_ports(&out_port, &in_port)?;

        // Do an initial run of the graph to connect and initialize all the components.
        // We'll have trace/stream metadata properties loaded and possibly some
        // events afterwards
        let last_run_status = graph.run_once()?;

        Ok(TraceIterator {
            _utils_plugin: utils_plugin,
            _ctf_plugin: ctf_plugin,
            _proxy_plugin: proxy_plugin,
            _ctf_src_class: ctf_src_class,
            _muxer_filter_class: muxer_filter_class,
            _proxy_sink_class: proxy_sink_class,
            _ctf_src: ctf_src,
            _ctf_params: ctf_params,
            _muxer_filter: muxer_filter,
            _proxy_sink: proxy_sink,
            graph,
            last_run_status,
            proxy_state,
        })
    }

    pub fn trace_properties(&self) -> &TraceProperties {
        &self.proxy_state.as_ref().trace_properties
    }

    pub fn stream_properties(&self) -> &BTreeSet<StreamProperties> {
        &self.proxy_state.as_ref().stream_properties
    }

    pub fn events_mut(&mut self) -> &mut VecDeque<OwnedEvent> {
        &mut self.proxy_state.as_mut().events
    }
}

impl Iterator for TraceIterator {
    type Item = BtResult<OwnedEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        // Drain the previous message iterators bunch of events
        if let Some(event) = self.proxy_state.as_mut().events.pop_front() {
            Some(Ok(event))
        } else {
            // Get another batch from upstream source component if not done
            match self.last_run_status {
                RunStatus::Ok | RunStatus::TryAgain => match self.graph.run_once() {
                    Ok(last_run_status) => {
                        self.last_run_status = last_run_status;
                        self.proxy_state.as_mut().events.pop_front().map(Ok)
                    }
                    Err(e) => Some(Err(e)),
                },
                RunStatus::End => None,
            }
        }
    }
}
