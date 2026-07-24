// SPDX-License-Identifier: MPL-2.0

use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

#[allow(dead_code)]
pub(crate) fn pretty_xml(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(input);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event()? {
            Event::Eof => break,

            // Remove existing indentation.
            Event::Text(text) if text.as_ref().iter().all(|byte| byte.is_ascii_whitespace()) => {}

            event => writer.write_event(event.into_owned())?,
        }
    }

    Ok(String::from_utf8(writer.into_inner())?)
}

pub(crate) fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
