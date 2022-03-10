#![deny(warnings, clippy::all)]

use babeltrace2_sys::internal_api::*;
use babeltrace2_sys::{Logger, LoggingLevel};
use std::fs;

fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn metadata_packet_decode() {
    init_logging();

    let td = tempfile::tempdir().unwrap();
    let metadata_path = td.path().join("metadata");
    fs::write(&metadata_path, METADATA).unwrap();

    let log_level = LoggingLevel::None;
    Logger::set_level(log_level);

    let cfg = PacketDecoderConfig {
        log_level,
        clock_class_offset_s: 0,
        clock_class_offset_ns: 0,
        force_clock_class_origin_unix_epoch: false,
        max_request_size: 64,
    };

    let mut dec = PacketDecoder::new(&metadata_path, &cfg).unwrap();

    let props = dec.packet_properties(&packets::A).unwrap().unwrap();
    log::debug!("{}", props);
    assert_eq!(
        props,
        PacketProperties {
            packet_total_size_bits: 512.into(),
            packet_content_size_bits: 512.into(),
            stream_class_id: 1.into(),
            data_stream_id: None,
            discarded_events: 0.into(),
            packet_seq_num: 1.into(),
            beginning_clock: 3.into(),
            end_clock: 4.into(),
        }
    );

    let props = dec.packet_properties(&packets::B).unwrap().unwrap();
    log::debug!("{}", props);
    assert_eq!(
        props,
        PacketProperties {
            packet_total_size_bits: 512.into(),
            packet_content_size_bits: 512.into(),
            stream_class_id: 1.into(),
            data_stream_id: None,
            discarded_events: 0.into(),
            packet_seq_num: 2.into(),
            beginning_clock: 5.into(),
            end_clock: 6.into(),
        }
    );

    let props = dec.packet_properties(&packets::C).unwrap().unwrap();
    log::debug!("{}", props);
    assert_eq!(
        props,
        PacketProperties {
            packet_total_size_bits: 512.into(),
            packet_content_size_bits: 512.into(),
            stream_class_id: 1.into(),
            data_stream_id: None,
            discarded_events: 0.into(),
            packet_seq_num: 4.into(),
            beginning_clock: 9.into(),
            end_clock: 10.into(),
        }
    );
}

#[test]
fn metadata_packet_decode_w_post_garbage() {
    init_logging();

    let td = tempfile::tempdir().unwrap();
    let metadata_path = td.path().join("metadata");
    fs::write(&metadata_path, METADATA).unwrap();

    let log_level = LoggingLevel::None;
    Logger::set_level(log_level);

    let cfg = PacketDecoderConfig {
        log_level,
        clock_class_offset_s: 0,
        clock_class_offset_ns: 0,
        force_clock_class_origin_unix_epoch: false,
        max_request_size: 64,
    };

    let mut data = packets::A.to_vec();
    data.extend_from_slice(&[0x65, 0x65, 0x6C, 0x20]);

    let mut dec = PacketDecoder::new(&metadata_path, &cfg).unwrap();

    let props = dec.packet_properties(&data).unwrap().unwrap();
    log::debug!("{}", props);
    assert_eq!(
        props,
        PacketProperties {
            packet_total_size_bits: 512.into(),
            packet_content_size_bits: 512.into(),
            stream_class_id: 1.into(),
            data_stream_id: None,
            discarded_events: 0.into(),
            packet_seq_num: 1.into(),
            beginning_clock: 3.into(),
            end_clock: 4.into(),
        }
    );

    let size_bytes = props.packet_total_size_bits.unwrap() as usize / 8;
    assert_eq!(&data[..size_bytes], &packets::A[..]);
}

#[test]
fn metadata_packet_decode_only_garbage() {
    init_logging();

    let td = tempfile::tempdir().unwrap();
    let metadata_path = td.path().join("metadata");
    fs::write(&metadata_path, METADATA).unwrap();

    let log_level = LoggingLevel::None;
    Logger::set_level(log_level);

    let cfg = PacketDecoderConfig {
        log_level,
        clock_class_offset_s: 0,
        clock_class_offset_ns: 0,
        force_clock_class_origin_unix_epoch: false,
        max_request_size: 64,
    };

    let data = vec![
        0x66, 0x65, 0x65, 0x6C, 0x20, 0x66, 0x65, 0x65, 0x6C, 0x20, 0x66, 0x65, 0x65, 0x6C, 0x20,
        0x66, 0x65, 0x65, 0x6C, 0x20,
    ];

    let mut dec = PacketDecoder::new(&metadata_path, &cfg).unwrap();

    assert!(dec.packet_properties(&data).is_err());
}

// Regen these from stream binary file: hexdump -ve '1/1 "0x%.2X, "' stream.bin
mod packets {
    pub const A: [u8; 64] = [
        0xC1, 0x1F, 0xFC, 0xC1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x49, 0x20, 0x66, 0x65, 0x65, 0x6C, 0x20, 0x73, 0x70, 0x65, 0x63, 0x69,
        0x61, 0x6C, 0x2E, 0x00,
    ];

    pub const B: [u8; 64] = [
        0xC1, 0x1F, 0xFC, 0xC1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
        0x00, 0x00, 0x00, 0x49, 0x20, 0x66, 0x65, 0x65, 0x6C, 0x20, 0x73, 0x70, 0x65, 0x63, 0x69,
        0x61, 0x6C, 0x2E, 0x00,
    ];

    pub const C: [u8; 64] = [
        0xC1, 0x1F, 0xFC, 0xC1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09,
        0x00, 0x00, 0x00, 0x49, 0x20, 0x66, 0x65, 0x65, 0x6C, 0x20, 0x73, 0x70, 0x65, 0x63, 0x69,
        0x61, 0x6C, 0x2E, 0x00,
    ];
}

const METADATA: &str = r#"/* CTF 1.8 */

/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2015-2020 Philippe Proulx <pproulx@efficios.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
 *
 * The following code was generated by barectf v3.1.0-dev
 * on 2022-03-10T05:26:24.809866.
 *
 * For more details, see <https://barectf.org/>.
 */

trace {
	major = 1;
	minor = 8;
	byte_order = le;
	packet.header := struct {
		integer {
			signed = false;
			size = 32;
			align = 32;
			byte_order = native;
			base = 10;
		} magic;
		integer {
			signed = false;
			size = 8;
			align = 8;
			byte_order = native;
			base = 10;
		} stream_id;
	} align(8);
};

env {
	domain = "bare";
	tracer_name = "barectf";
	tracer_major = 3;
	tracer_minor = 1;
	tracer_patch = 0;
	tracer_pre = "-dev";
	barectf_gen_date = "2022-03-10T05:26:24.809866";
};

clock {
	name = default;
	freq = 1000000000;
	precision = 0;
	offset_s = 0;
	offset = 0;
	absolute = false;
};

/* Data stream type `d2` */
stream {
	id = 0;
	packet.context := struct {
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} packet_size;
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} content_size;
		integer {
			signed = false;
			size = 64;
			align = 64;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp_begin;
		integer {
			signed = false;
			size = 64;
			align = 64;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp_end;
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} events_discarded;
		integer {
			signed = false;
			size = 32;
			align = 32;
			byte_order = native;
			base = 10;
		} packet_seq_num;
	} align(8);
	event.header := struct {
		integer {
			signed = false;
			size = 8;
			align = 8;
			byte_order = native;
			base = 10;
		} id;
		integer {
			signed = false;
			size = 32;
			align = 32;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp;
	} align(8);
};

event {
	stream_id = 0;
	id = 0;
	name = "ev2";
	fields := struct {
		string {
			encoding = UTF8;
		} s;
	} align(1);
};

/* Data stream type `default` */
stream {
	id = 1;
	packet.context := struct {
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} packet_size;
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} content_size;
		integer {
			signed = false;
			size = 64;
			align = 64;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp_begin;
		integer {
			signed = false;
			size = 64;
			align = 64;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp_end;
		integer {
			signed = false;
			size = 16;
			align = 16;
			byte_order = native;
			base = 10;
		} events_discarded;
		integer {
			signed = false;
			size = 32;
			align = 32;
			byte_order = native;
			base = 10;
		} packet_seq_num;
	} align(8);
	event.header := struct {
		integer {
			signed = false;
			size = 8;
			align = 8;
			byte_order = native;
			base = 10;
		} id;
		integer {
			signed = false;
			size = 32;
			align = 32;
			byte_order = native;
			base = 10;
			map = clock.default.value;
		} timestamp;
	} align(8);
};

event {
	stream_id = 1;
	id = 0;
	name = "ev";
	fields := struct {
		string {
			encoding = UTF8;
		} s;
	} align(1);
};"#;
