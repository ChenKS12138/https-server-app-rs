mod fsm;

use rust_fsm::*;

#[derive(Clone, Debug)]
pub struct FormDataPart {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

impl FormDataPart {
    fn new() -> Self {
        Self {
            content_type: None,
            data: Vec::new(),
            filename: None,
            name: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormData {
    pub boundary: Vec<u8>,
    pub parts: Vec<FormDataPart>,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            boundary: Vec::new(),
            parts: Vec::new(),
        }
    }
    pub fn get_part(&self, name: &str) -> Option<&FormDataPart> {
        for part in &self.parts {
            if let Some(part_name) = &part.name {
                if part_name == name {
                    return Some(&part);
                }
            }
        }
        None
    }
    pub fn parse(raw: &[u8]) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let mut machine: StateMachine<fsm::FormData> = StateMachine::new();
        let mut form_data = FormData::new();
        let mut form_data_part = FormDataPart::new();
        let mut header_field: Vec<u8> = Vec::new();
        let mut header_value: Vec<u8> = Vec::new();
        let mut boundary_like: Vec<u8> = Vec::new();

        for byte in raw.iter() {
            let byte = byte.clone();
            let effect = machine.consume(match byte {
                b'-' if machine.state() != &fsm::FormDataState::Boundary
                    && machine.state() != &fsm::FormDataState::End
                    && boundary_like == form_data.boundary =>
                {
                    &fsm::FormDataInput::EndDash
                }
                _ if (machine.state() == &fsm::FormDataState::Data
                    || machine.state() == &fsm::FormDataState::DataToBoundary)
                    && form_data.boundary[boundary_like.len()] == byte =>
                {
                    &fsm::FormDataInput::BoundaryLike
                }
                b'-' => &fsm::FormDataInput::Dash,
                b'\r' => &fsm::FormDataInput::Cr,
                b'\n' => &fsm::FormDataInput::Lf,
                b':' => &fsm::FormDataInput::Colon,
                b' ' => &fsm::FormDataInput::Blank,
                _ => &fsm::FormDataInput::Alpha,
            })?;

            match effect {
                Some(effect) => match effect {
                    fsm::FormDataOutput::EffectAppendBoundary => {
                        form_data.boundary.push(byte);
                    }
                    fsm::FormDataOutput::EffectAppendData => {
                        if !boundary_like.is_empty() {
                            form_data_part.data.append(&mut boundary_like);
                        }
                        form_data_part.data.push(byte);
                    }
                    fsm::FormDataOutput::EffectAppendHeader => {
                        let mut field = Vec::new();
                        field.append(&mut header_field);
                        let header_field = String::from_utf8(field)?;

                        let mut value = Vec::new();
                        value.append(&mut header_value);

                        match header_field.as_str() {
                            "Content-Disposition" => {
                                let header_value = String::from_utf8(value)?;
                                let pairs: Vec<Vec<&str>> = header_value
                                    .split("; ")
                                    .map(|item| -> Vec<&str> { item.splitn(2, "=").collect() })
                                    .collect();
                                for pair in pairs {
                                    if pair.len() >= 2 {
                                        match pair[0] {
                                            "name" => {
                                                form_data_part.name = Some(String::from(
                                                    &pair[1][1..(pair[1].len() - 1)],
                                                ));
                                            }
                                            "filename" => {
                                                form_data_part.filename = Some(String::from(
                                                    &pair[1][1..(pair[1].len() - 1)],
                                                ));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            "Content-Type" => {
                                form_data_part.content_type = Some(String::from_utf8(value)?);
                            }
                            _ => {}
                        }
                    }
                    fsm::FormDataOutput::EffectAppendHeaderField => {
                        header_field.push(byte);
                    }
                    fsm::FormDataOutput::EffectAppendHeaderValue => {
                        header_value.push(byte);
                    }
                    fsm::FormDataOutput::EffectAppendLikeBoundary => {
                        boundary_like.push(byte);
                    }
                    fsm::FormDataOutput::EffectAppendPart => {
                        form_data.parts.push(form_data_part.clone());
                    }
                    fsm::FormDataOutput::EffectFormData => {
                        machine.consume(&fsm::FormDataInput::End).unwrap();
                        return Ok(Some(form_data));
                    }
                },
                None => {
                    // do nothing
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::FormData;
    /// Example
    /// ------WebKitFormBoundarype6X79pAiSSGJJKV
    /// Content-Disposition: form-data; name="file"; filename="aria2.conf"
    /// Content-Type: application/octet-stream
    ///
    /// # rpc-user=chenks
    /// # rpc-passwd=749923710
    /// rpc-secret=token
    /// enable-rpc=true
    /// rpc-allow-origin-all=true
    /// rpc-listen-all=true
    /// max-concurrent-downloads=5
    /// continue=true
    /// max-connection-per-server=5
    /// min-split-size=10M
    /// split=10
    /// max-overall-download-limit=0
    /// max-download-limit=0
    /// max-overall-upload-limit=0
    /// max-upload-limit=0
    /// dir=/Users/brucezhou/movie
    /// file-allocation=prealloc
    /// ------WebKitFormBoundarype6X79pAiSSGJJKV--
    #[test]
    fn test_form_date_parse() {
        let data = vec![
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117,
            110, 100, 97, 114, 121, 112, 101, 54, 88, 55, 57, 112, 65, 105, 83, 83, 71, 74, 74, 75,
            86, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105,
            116, 105, 111, 110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97,
            109, 101, 61, 34, 102, 105, 108, 101, 34, 59, 32, 102, 105, 108, 101, 110, 97, 109,
            101, 61, 34, 97, 114, 105, 97, 50, 46, 99, 111, 110, 102, 34, 13, 10, 67, 111, 110,
            116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116,
            105, 111, 110, 47, 111, 99, 116, 101, 116, 45, 115, 116, 114, 101, 97, 109, 13, 10, 13,
            10, 35, 32, 114, 112, 99, 45, 117, 115, 101, 114, 61, 99, 104, 101, 110, 107, 115, 10,
            35, 32, 114, 112, 99, 45, 112, 97, 115, 115, 119, 100, 61, 55, 52, 57, 57, 50, 51, 55,
            49, 48, 10, 114, 112, 99, 45, 115, 101, 99, 114, 101, 116, 61, 116, 111, 107, 101, 110,
            10, 101, 110, 97, 98, 108, 101, 45, 114, 112, 99, 61, 116, 114, 117, 101, 10, 114, 112,
            99, 45, 97, 108, 108, 111, 119, 45, 111, 114, 105, 103, 105, 110, 45, 97, 108, 108, 61,
            116, 114, 117, 101, 10, 114, 112, 99, 45, 108, 105, 115, 116, 101, 110, 45, 97, 108,
            108, 61, 116, 114, 117, 101, 10, 109, 97, 120, 45, 99, 111, 110, 99, 117, 114, 114,
            101, 110, 116, 45, 100, 111, 119, 110, 108, 111, 97, 100, 115, 61, 53, 10, 99, 111,
            110, 116, 105, 110, 117, 101, 61, 116, 114, 117, 101, 10, 109, 97, 120, 45, 99, 111,
            110, 110, 101, 99, 116, 105, 111, 110, 45, 112, 101, 114, 45, 115, 101, 114, 118, 101,
            114, 61, 53, 10, 109, 105, 110, 45, 115, 112, 108, 105, 116, 45, 115, 105, 122, 101,
            61, 49, 48, 77, 10, 115, 112, 108, 105, 116, 61, 49, 48, 10, 109, 97, 120, 45, 111,
            118, 101, 114, 97, 108, 108, 45, 100, 111, 119, 110, 108, 111, 97, 100, 45, 108, 105,
            109, 105, 116, 61, 48, 10, 109, 97, 120, 45, 100, 111, 119, 110, 108, 111, 97, 100, 45,
            108, 105, 109, 105, 116, 61, 48, 10, 109, 97, 120, 45, 111, 118, 101, 114, 97, 108,
            108, 45, 117, 112, 108, 111, 97, 100, 45, 108, 105, 109, 105, 116, 61, 48, 10, 109, 97,
            120, 45, 117, 112, 108, 111, 97, 100, 45, 108, 105, 109, 105, 116, 61, 48, 10, 100,
            105, 114, 61, 47, 85, 115, 101, 114, 115, 47, 98, 114, 117, 99, 101, 122, 104, 111,
            117, 47, 109, 111, 118, 105, 101, 10, 102, 105, 108, 101, 45, 97, 108, 108, 111, 99,
            97, 116, 105, 111, 110, 61, 112, 114, 101, 97, 108, 108, 111, 99, 13, 10, 45, 45, 45,
            45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117, 110, 100, 97,
            114, 121, 112, 101, 54, 88, 55, 57, 112, 65, 105, 83, 83, 71, 74, 74, 75, 86, 45, 45,
            13, 10,
        ];
        let form_data = FormData::parse(&data);
        let form_data = form_data.unwrap().unwrap();
        assert_eq!(
            form_data
                .get_part("file")
                .unwrap()
                .filename
                .as_ref()
                .unwrap(),
            "aria2.conf"
        );
        assert_eq!(
            form_data
                .get_part("file")
                .unwrap()
                .content_type
                .as_ref()
                .unwrap(),
            "application/octet-stream"
        );
        assert_eq!(
            form_data.get_part("file").unwrap().data,
            vec![
                35, 32, 114, 112, 99, 45, 117, 115, 101, 114, 61, 99, 104, 101, 110, 107, 115, 10,
                35, 32, 114, 112, 99, 45, 112, 97, 115, 115, 119, 100, 61, 55, 52, 57, 57, 50, 51,
                55, 49, 48, 10, 114, 112, 99, 45, 115, 101, 99, 114, 101, 116, 61, 116, 111, 107,
                101, 110, 10, 101, 110, 97, 98, 108, 101, 45, 114, 112, 99, 61, 116, 114, 117, 101,
                10, 114, 112, 99, 45, 97, 108, 108, 111, 119, 45, 111, 114, 105, 103, 105, 110, 45,
                97, 108, 108, 61, 116, 114, 117, 101, 10, 114, 112, 99, 45, 108, 105, 115, 116,
                101, 110, 45, 97, 108, 108, 61, 116, 114, 117, 101, 10, 109, 97, 120, 45, 99, 111,
                110, 99, 117, 114, 114, 101, 110, 116, 45, 100, 111, 119, 110, 108, 111, 97, 100,
                115, 61, 53, 10, 99, 111, 110, 116, 105, 110, 117, 101, 61, 116, 114, 117, 101, 10,
                109, 97, 120, 45, 99, 111, 110, 110, 101, 99, 116, 105, 111, 110, 45, 112, 101,
                114, 45, 115, 101, 114, 118, 101, 114, 61, 53, 10, 109, 105, 110, 45, 115, 112,
                108, 105, 116, 45, 115, 105, 122, 101, 61, 49, 48, 77, 10, 115, 112, 108, 105, 116,
                61, 49, 48, 10, 109, 97, 120, 45, 111, 118, 101, 114, 97, 108, 108, 45, 100, 111,
                119, 110, 108, 111, 97, 100, 45, 108, 105, 109, 105, 116, 61, 48, 10, 109, 97, 120,
                45, 100, 111, 119, 110, 108, 111, 97, 100, 45, 108, 105, 109, 105, 116, 61, 48, 10,
                109, 97, 120, 45, 111, 118, 101, 114, 97, 108, 108, 45, 117, 112, 108, 111, 97,
                100, 45, 108, 105, 109, 105, 116, 61, 48, 10, 109, 97, 120, 45, 117, 112, 108, 111,
                97, 100, 45, 108, 105, 109, 105, 116, 61, 48, 10, 100, 105, 114, 61, 47, 85, 115,
                101, 114, 115, 47, 98, 114, 117, 99, 101, 122, 104, 111, 117, 47, 109, 111, 118,
                105, 101, 10, 102, 105, 108, 101, 45, 97, 108, 108, 111, 99, 97, 116, 105, 111,
                110, 61, 112, 114, 101, 97, 108, 108, 111, 99, 13, 10,
            ]
        )
    }
}
