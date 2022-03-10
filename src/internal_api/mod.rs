use crate::{ffi, BtResult, BtResultExt, Error, LoggingLevel};
use std::convert::{AsMut, AsRef};
use std::ffi::{c_void, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{cmp, fmt, mem, ptr};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PacketProperties {
    pub packet_total_size_bits: Option<u64>,
    pub packet_content_size_bits: Option<u64>,
    pub stream_class_id: Option<u64>,
    pub data_stream_id: Option<u64>,
    pub discarded_events: Option<u64>,
    pub packet_seq_num: Option<u64>,
    pub beginning_clock: Option<u64>,
    pub end_clock: Option<u64>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PacketDecoderConfig {
    pub log_level: LoggingLevel,
    pub clock_class_offset_s: i64,
    pub clock_class_offset_ns: i64,
    pub force_clock_class_origin_unix_epoch: bool,
    pub max_request_size: usize,
}

impl Default for PacketDecoderConfig {
    fn default() -> Self {
        PacketDecoderConfig {
            log_level: LoggingLevel::None,
            clock_class_offset_s: 0,
            clock_class_offset_ns: 0,
            force_clock_class_origin_unix_epoch: false,
            max_request_size: 4096,
        }
    }
}

pub struct PacketDecoder {
    md_dec: *mut ffi::ctf_metadata_decoder,
    msg_iter: *mut ffi::ctf_msg_iter,
    state: BoxedRawMsgIterState,
}

impl PacketDecoder {
    pub fn new<P: AsRef<Path>>(metadata_path: P, config: &PacketDecoderConfig) -> BtResult<Self> {
        let md_path = metadata_path.as_ref();
        if !md_path.exists() {
            return Err(Error::NonExistentMetadataPath(
                md_path.to_string_lossy().into(),
            ));
        }
        if !md_path.is_file() {
            return Err(Error::MetadataPathNotFile(md_path.to_string_lossy().into()));
        }
        let md_path = CString::new(md_path.as_os_str().as_bytes())?;
        let md_file_opts = CString::new("rb")?;
        let md_file = unsafe { libc::fopen(md_path.as_ptr(), md_file_opts.as_c_str().as_ptr()) };
        if md_file.is_null() {
            return Err(Error::MetadataFileOpen(md_path.to_string_lossy().into()));
        }

        let mut state = BoxedRawMsgIterState::new_null();

        // Forge a component class and component for use by the decoder and msg-iter
        let name = CString::new("forged-msg-iter")?;
        state.as_mut().comp_class.name = unsafe { ffi::g_string_new(name.as_c_str().as_ptr()) };
        state.as_mut().comp_class.plugin_name =
            unsafe { ffi::g_string_new(name.as_c_str().as_ptr()) };
        state.as_mut().comp_class.type_ =
            ffi::bt_component_class_type::BT_COMPONENT_CLASS_TYPE_SOURCE;
        state.as_mut().comp_class.base.is_shared = true;
        state.as_mut().comp_class.base.ref_count = 1;
        let mut comp: *mut ffi::bt_component = ptr::null_mut();
        let comp_status = unsafe {
            ffi::bt_component_create(
                &mut state.as_mut().comp_class,
                name.as_c_str().as_ptr(),
                config.log_level.into(),
                &mut comp,
            )
        };
        comp_status.capi_result()?;
        if comp.is_null() {
            log::error!("Could not forge a new bt_component from source component class");
            return Err(Error::Memory);
        }

        let md_cfg = ffi::ctf_metadata_decoder_config {
            log_level: config.log_level.into(),
            self_comp: comp as *mut ffi::bt_self_component, // bt_self_t is a cast of bt_t
            clock_class_offset_s: config.clock_class_offset_s,
            clock_class_offset_ns: config.clock_class_offset_ns,
            force_clock_class_origin_unix_epoch: config.force_clock_class_origin_unix_epoch,
            create_trace_class: true,
            keep_plain_text: false,
        };

        let md_dec = unsafe { ffi::ctf_metadata_decoder_create(&md_cfg) };
        if md_dec.is_null() {
            return Err(Error::CtfMetadataDecoderCreate);
        }

        // Process the metadata content
        let md_status = unsafe { ffi::ctf_metadata_decoder_append_content(md_dec, md_file) };
        if md_status != ffi::ctf_metadata_decoder_status::CTF_METADATA_DECODER_STATUS_OK {
            return Err(Error::CtfMetadataDecoderStatus(md_status as _));
        }

        let tc = unsafe { ffi::ctf_metadata_decoder_get_ir_trace_class(md_dec) };
        if tc.is_null() {
            log::error!("Could not get CTF metadata decoder IR trace class");
            return Err(Error::Memory);
        }

        let ctf_tc = unsafe { ffi::ctf_metadata_decoder_borrow_ctf_trace_class(md_dec) };
        if ctf_tc.is_null() {
            log::error!("Could not borrow CTF metadata decoder CTF trace class");
            return Err(Error::ResourceBorrow);
        }

        // All done with the metadata file
        unsafe { libc::fclose(md_file) };

        let trace = unsafe { ffi::bt_trace_create(tc) };
        if trace.is_null() {
            log::error!("Could not create a trace instance using the CTF metadata trace class");
            return Err(Error::Memory);
        }

        let msg_iter_med_opts = ffi::ctf_msg_iter_medium_ops {
            request_bytes: Some(msg_iter_request_bytes),
            seek: None,
            switch_packet: Some(msg_iter_switch_packet),
            borrow_stream: Some(msg_iter_borrow_stream),
        };

        state.set_comp_trace(comp, trace);

        let msg_iter = unsafe {
            ffi::ctf_msg_iter_create(
                ctf_tc,
                config.max_request_size as _,
                msg_iter_med_opts,
                state.as_raw() as *mut c_void,
                config.log_level.into(),
                comp as *mut ffi::bt_self_component, // bt_self_t is a cast of bt_t
                ptr::null_mut(),
            )
        };
        if msg_iter.is_null() {
            return Err(Error::CtfMessageIterCreate);
        }

        // Don't allocate objects since we're just parsing packet header contents
        unsafe { ffi::ctf_msg_iter_set_dry_run(msg_iter, true) };

        Ok(Self {
            md_dec,
            msg_iter,
            state,
        })
    }

    pub fn packet_properties(&mut self, packet: &[u8]) -> BtResult<Option<PacketProperties>> {
        unsafe { ffi::ctf_msg_iter_reset(self.msg_iter) };
        self.state.set_buf(packet);
        let mut props = ffi::ctf_msg_iter_packet_properties {
            exp_packet_total_size: 0,
            exp_packet_content_size: 0,
            stream_class_id: 0,
            data_stream_id: 0,
            snapshots: ffi::ctf_msg_iter_packet_properties__bindgen_ty_1 {
                discarded_events: 0,
                packets: 0,
                beginning_clock: 0,
                end_clock: 0,
            },
        };
        let status = unsafe { ffi::ctf_msg_iter_get_packet_properties(self.msg_iter, &mut props) };
        self.state.reset();
        match status {
            ffi::ctf_msg_iter_status::CTF_MSG_ITER_STATUS_OK => Ok(Some(props.into())),
            ffi::ctf_msg_iter_status::CTF_MSG_ITER_STATUS_EOF
            | ffi::ctf_msg_iter_status::CTF_MSG_ITER_STATUS_AGAIN => Ok(None),
            _ => Err(Error::Failure(status as _)),
        }
    }
}

impl Drop for PacketDecoder {
    fn drop(&mut self) {
        unsafe {
            ffi::ctf_msg_iter_destroy(self.msg_iter);
            ffi::ctf_metadata_decoder_destroy(self.md_dec);
        }
    }
}

struct MsgIterState {
    comp_class: ffi::bt_component_class,
    comp: *mut ffi::bt_component,
    trace: *mut ffi::bt_trace,
    read_index: usize,
    packet: *const u8,
    packet_size: usize,
}

struct BoxedRawMsgIterState(*mut MsgIterState);

impl BoxedRawMsgIterState {
    fn new_null() -> Self {
        BoxedRawMsgIterState(Box::into_raw(Box::new(MsgIterState {
            trace: ptr::null_mut(),
            comp_class: unsafe { mem::zeroed() },
            comp: ptr::null_mut(),
            read_index: 0,
            packet: ptr::null(),
            packet_size: 0,
        })))
    }

    fn as_raw(&mut self) -> *mut MsgIterState {
        self.0
    }

    fn set_comp_trace(&mut self, comp: *mut ffi::bt_component, trace: *mut ffi::bt_trace) {
        self.as_mut().comp = comp;
        self.as_mut().trace = trace;
    }

    fn reset(&mut self) {
        self.as_mut().read_index = 0;
        self.as_mut().packet = ptr::null();
        self.as_mut().packet_size = 0;
    }

    fn set_buf(&mut self, packet: &[u8]) {
        self.reset();
        self.as_mut().packet = packet.as_ptr();
        self.as_mut().packet_size = packet.len();
    }
}

impl AsRef<MsgIterState> for BoxedRawMsgIterState {
    fn as_ref(&self) -> &MsgIterState {
        unsafe { &(*self.0) }
    }
}

impl AsMut<MsgIterState> for BoxedRawMsgIterState {
    fn as_mut(&mut self) -> &mut MsgIterState {
        unsafe { &mut (*self.as_raw()) }
    }
}

impl Drop for BoxedRawMsgIterState {
    fn drop(&mut self) {
        debug_assert!(!self.0.is_null());
        unsafe {
            debug_assert!(!self.as_ref().trace.is_null());
            ffi::bt_trace_put_ref(self.as_ref().trace);
            debug_assert!(!self.as_ref().comp.is_null());
            ffi::bt_component_put_ref(self.as_ref().comp);
            ffi::bt_current_thread_clear_error();
            drop(Box::from_raw(self.0));
        }
    }
}

/// Returns the next byte buffer to be used by the binary file
/// reader to deserialize binary data
extern "C" fn msg_iter_request_bytes(
    request_sz: ffi::size_t,
    buffer_addr: *mut *mut u8,
    buffer_sz: *mut ffi::size_t,
    data: *mut c_void,
) -> ffi::ctf_msg_iter_medium_status::Type {
    if data.is_null() {
        log::error!("CTF message iterator state is NULL");
        return ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_ERROR;
    }
    let state_raw = data as *mut MsgIterState;
    let state = unsafe { &mut (*state_raw) };

    if state.packet.is_null() {
        unsafe {
            *buffer_addr = ptr::null_mut();
            *buffer_sz = 0;
        }
        ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_ERROR
    } else if state.packet_size == 0 {
        unsafe {
            *buffer_addr = ptr::null_mut();
            *buffer_sz = 0;
        }
        ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_AGAIN
    } else if state.read_index == state.packet_size {
        unsafe {
            *buffer_addr = ptr::null_mut();
            *buffer_sz = 0;
        }
        ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_EOF
    } else {
        debug_assert!(state.packet_size > state.read_index);
        let max_len = cmp::min(request_sz as usize, state.packet_size - state.read_index);
        let start = unsafe { state.packet.add(state.read_index) };
        unsafe {
            *buffer_addr = start as *mut _; // msg_iter doesn't modify buffer but isn't const in the decl
            *buffer_sz = max_len as _;
        }
        ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_OK
    }
}

/// Called when the message iterator wishes to inform the medium that it is about to start a new packet.
///
/// After the iterator has called switch_packet, the following call to
/// request_bytes must return the content at the start of the next packet.
extern "C" fn msg_iter_switch_packet(data: *mut c_void) -> ffi::ctf_msg_iter_medium_status::Type {
    if data.is_null() {
        log::error!("CTF message iterator state is NULL");
        return ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_ERROR;
    }
    ffi::ctf_msg_iter_medium_status::CTF_MSG_ITER_MEDIUM_STATUS_OK
}

/// Returns a stream instance (weak reference) for the given stream class.
///
/// This is called after a packet header is read, and the corresponding
/// stream class is found by the message iterator.
extern "C" fn msg_iter_borrow_stream(
    _stream_class: *mut ffi::bt_stream_class,
    _stream_id: i64,
    _data: *mut c_void,
) -> *mut ffi::bt_stream {
    // Not used
    ptr::null_mut()
}

impl From<ffi::ctf_msg_iter_packet_properties> for PacketProperties {
    fn from(p: ffi::ctf_msg_iter_packet_properties) -> Self {
        PacketProperties {
            packet_total_size_bits: if p.exp_packet_total_size >= 0 {
                Some(p.exp_packet_total_size as _)
            } else {
                None
            },
            packet_content_size_bits: if p.exp_packet_content_size >= 0 {
                Some(p.exp_packet_content_size as _)
            } else {
                None
            },
            stream_class_id: if p.stream_class_id != u64::MAX {
                Some(p.stream_class_id)
            } else {
                None
            },
            data_stream_id: if p.data_stream_id >= 0 {
                Some(p.data_stream_id as _)
            } else {
                None
            },
            discarded_events: if p.snapshots.discarded_events != u64::MAX {
                Some(p.snapshots.discarded_events)
            } else {
                None
            },
            packet_seq_num: if p.snapshots.packets != u64::MAX {
                Some(p.snapshots.packets)
            } else {
                None
            },
            beginning_clock: if p.snapshots.beginning_clock != u64::MAX {
                Some(p.snapshots.beginning_clock)
            } else {
                None
            },
            end_clock: if p.snapshots.end_clock != u64::MAX {
                Some(p.snapshots.end_clock)
            } else {
                None
            },
        }
    }
}

impl fmt::Display for PacketProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{stream_id={}, packet_size={}, content_size={}, clock_begin={}, clock_end={}, discarded={}, seq_num={}}}",
            DisaplayableOptu64(&self.stream_class_id),
            DisaplayableOptu64(&self.packet_total_size_bits),
            DisaplayableOptu64(&self.packet_content_size_bits),
            DisaplayableOptu64(&self.beginning_clock),
            DisaplayableOptu64(&self.end_clock),
            DisaplayableOptu64(&self.discarded_events),
            DisaplayableOptu64(&self.packet_seq_num),
        )
    }
}

struct DisaplayableOptu64<'a>(&'a Option<u64>);

impl<'a> fmt::Display for DisaplayableOptu64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = self.0 {
            t.fmt(f)
        } else {
            f.write_str("NA")
        }
    }
}
