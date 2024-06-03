use crate::{
    BtResult, ComponentClassFilter, ComponentClassSink, ComponentClassSource, ComponentFilter,
    ComponentSink, ComponentSource, CtfPlugin, CtfPluginSinkExt, CtfPluginSinkFsInitParams, Graph,
    Logger, LoggingLevel, Plugin, SourcePluginDescriptor, SourcePluginState, UtilsPlugin,
};
use std::os::raw::c_void;
use tracing::debug;

pub struct EncoderPipeline {
    _utils_plugin: UtilsPlugin,
    _ctf_plugin: CtfPlugin,
    _user_plugin: Plugin,
    _user_src_class: ComponentClassSource,
    _muxer_filter_class: ComponentClassFilter,
    _ctf_sink_class: ComponentClassSink,
    _user_src: ComponentSource,
    _muxer_filter: ComponentFilter,
    _ctf_sink: ComponentSink,
    pub graph: Graph,
}

impl EncoderPipeline {
    pub fn new<P: SourcePluginDescriptor>(
        log_level: LoggingLevel,
        plugin_state: Box<SourcePluginState>,
        ctf_params: &CtfPluginSinkFsInitParams,
    ) -> BtResult<Self> {
        Logger::set_level(log_level);

        // Load builtin plugins we need
        let utils_plugin = UtilsPlugin::load()?;
        let ctf_plugin = CtfPlugin::load()?;

        // Load user plugin
        let user_plugin = P::load()?;

        // Borrow the component classes from the plugins
        let output_name = P::output_name();
        let user_src_class = user_plugin.borrow_source_component_class_by_name(output_name)?;
        let muxer_filter_class = utils_plugin.borrow_muxer_filter_component_class()?;
        let ctf_sink_class = ctf_plugin
            .borrow_sink_component_class_by_name(ctf_params.sink_component_class_name())?;

        let mut graph = Graph::new()?;

        // Add the components to the processing graph
        let user_src = graph.add_source_component_with_initialize_method_data(
            &user_src_class,
            P::graph_node_name(),
            Box::into_raw(plugin_state) as *mut c_void,
            log_level,
        )?;

        let muxer_filter = graph.add_filter_component(
            &muxer_filter_class,
            UtilsPlugin::graph_node_name(),
            log_level,
        )?;

        let ctf_sink = graph.add_sink_component(
            &ctf_sink_class,
            CtfPlugin::sink_graph_node_name(),
            ctf_params.parameters(),
            log_level,
        )?;

        // Connect source port to the muxer filter input port
        debug!("Connecting user source port(s) to muxer");
        let out_port = user_src.borrow_output_port_by_index(0)?;
        let in_port = muxer_filter.borrow_input_port_by_index(0)?;
        graph.connect_ports(&out_port, &in_port)?;

        // Connect the mux'd filter output port to the ctf sink input port
        let out_port = muxer_filter.borrow_output_port_by_index(0)?;
        let in_port = ctf_sink.borrow_input_port_by_index(0)?;
        graph.connect_ports(&out_port, &in_port)?;

        Ok(EncoderPipeline {
            _utils_plugin: utils_plugin,
            _ctf_plugin: ctf_plugin,
            _user_plugin: user_plugin,
            _user_src_class: user_src_class,
            _muxer_filter_class: muxer_filter_class,
            _ctf_sink_class: ctf_sink_class,
            _user_src: user_src,
            _muxer_filter: muxer_filter,
            _ctf_sink: ctf_sink,
            graph,
        })
    }
}
