pub mod tlv;
use crate::{encoding, length, ZVTError, ZVTResult, Zvt};

/// From feig, 6.13
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x04, instr = 0x0c)]
pub struct RequestForData {
    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::WriteData>,
}

/// Custom length parsing for temperatures.
///
/// The specification says that the temperature has always the length of four
/// bytes. However, we observed that for low temperatures the terminals return
/// only three bytes. We implement a custom decoder which decodes both lengths.
/// This can be removed once the bug is fixed. See
/// https://groups.google.com/a/qwello.eu/g/embedded-team/c/Ma5XyYzkdTQ/m/ZrSLp1tVAAAJ
/// for more details.
struct Temperature {}

impl length::Length for Temperature {
    fn deserialize(bytes: &[u8]) -> ZVTResult<(usize, &[u8])> {
        if bytes.len() < 3 {
            return Err(ZVTError::IncompleteData);
        }
        let len = std::cmp::min(bytes.len(), 4);
        Ok((len, bytes))
    }

    fn serialize(_len: usize) -> Vec<u8> {
        vec![]
    }
}

/// From Feig manual, 6.3 Enhanced system information.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x06, instr = 0x0f)]
pub struct CVendFunctionsEnhancedSystemInformationCompletion {
    #[zvt_bmp(length  = length::Fixed<8>)]
    pub device_id: String,

    #[zvt_bmp(length = length::Fixed<17>)]
    pub sw_version: String,

    #[zvt_bmp(length  = length::Fixed<8>)]
    pub terminal_id: String,

    #[zvt_bmp(length = Temperature)]
    pub temperature: String,
}

/// From Feig specific manual, 6.13 - Write File.
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x08, instr = 0x14)]
pub struct WriteFile {
    #[zvt_bmp(length = length::Fixed<3>, encoding = encoding::Bcd)]
    pub password: usize,

    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::WriteFile>,
}

/// Feig, 5.1
#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x0f, instr = 0xa1)]
pub struct CVendFunctions {
    #[zvt_bmp( encoding = encoding::BigEndian)]
    pub instr: u16,
}

#[derive(Debug, PartialEq, Zvt)]
#[zvt_control_field(class = 0x80, instr = 0x00)]
pub struct WriteData {
    #[zvt_bmp(number = 0x06, length = length::Tlv)]
    pub tlv: Option<tlv::WriteData>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::packets::tests::get_bytes;
    use crate::ZvtSerializer;

    #[test]
    fn test_request_for_data() {
        let bytes = get_bytes("1682080275.777628000_192.168.0.59_192.168.0.139.blob");
        let expected = RequestForData {
            tlv: Some(tlv::WriteData {
                file: Some(tlv::File {
                    file_id: Some(0x23),
                    file_offset: Some(65000),
                    file_size: None,
                    payload: None,
                }),
            }),
        };
        assert_eq!(RequestForData::zvt_deserialize(&bytes).unwrap().0, expected);
        assert_eq!(bytes, expected.zvt_serialize());
    }

    #[test]
    fn test_cv_end_functions_enhanced_systems_information_completion() {
        let bytes = get_bytes("1680761818.768770000_pt_ecr.blob");
        let expected = CVendFunctionsEnhancedSystemInformationCompletion {
            device_id: "17FD1E3C".to_string(),
            sw_version: "GER-APP-v2.0.9   ".to_string(),
            terminal_id: "52523535".to_string(),
            temperature: "24.4".to_string(),
        };
        assert_eq!(
            CVendFunctionsEnhancedSystemInformationCompletion::zvt_deserialize(&bytes)
                .unwrap()
                .0,
            expected
        );
        assert_eq!(bytes, expected.zvt_serialize());

        // Test the temperature encoding bug.
        let bytes = b"\x06\x0f$17FE5C90GER-APP-v2.0.9   525251118.0";
        let expected = CVendFunctionsEnhancedSystemInformationCompletion {
            device_id: "17FE5C90".to_string(),
            sw_version: "GER-APP-v2.0.9   ".to_string(),
            terminal_id: "52525111".to_string(),
            temperature: "8.0".to_string(),
        };

        assert_eq!(
            CVendFunctionsEnhancedSystemInformationCompletion::zvt_deserialize(bytes)
                .unwrap()
                .0,
            expected
        );
    }

    #[test]
    fn test_write_file() {
        let bytes = get_bytes("1682080275.594788000_192.168.0.139_192.168.0.59.blob");
        let expected = WriteFile {
            password: 123456,
            tlv: Some(tlv::WriteFile {
                files: vec![
                    tlv::File {
                        file_id: Some(0x23),
                        file_size: Some(3357255),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x22),
                        file_size: Some(3611),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x12),
                        file_size: Some(125825),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x10),
                        file_size: Some(3479044),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x11),
                        file_size: Some(10909539),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x13),
                        file_size: Some(1068),
                        file_offset: None,
                        payload: None,
                    },
                    tlv::File {
                        file_id: Some(0x14),
                        file_size: Some(1160),
                        file_offset: None,
                        payload: None,
                    },
                ],
            }),
        };
        assert_eq!(WriteFile::zvt_deserialize(&bytes).unwrap().0, expected);
        assert_eq!(bytes, expected.zvt_serialize());
    }

    #[test]
    fn test_cvend_functions() {
        let bytes = get_bytes("1680761818.690979000_ecr_pt.blob");
        let expected = CVendFunctions { instr: 0x01 };
        assert_eq!(CVendFunctions::zvt_deserialize(&bytes).unwrap().0, expected);
        assert_eq!(bytes, expected.zvt_serialize());
    }

    #[test]
    fn test_write_data() {
        let bytes = get_bytes("1682080310.907262000_192.168.0.139_192.168.0.59.blob");
        let dummy_data = vec![0; 0x042c];
        let expected = WriteData {
            tlv: Some(tlv::WriteData {
                file: Some(tlv::File {
                    file_id: Some(0x13),
                    file_offset: Some(0),
                    file_size: None,
                    payload: Some(dummy_data.clone()), // dummy data.
                }),
            }),
        };
        let mut actual = WriteData::zvt_deserialize(&bytes).unwrap().0;
        let file = actual.tlv.as_mut().unwrap().file.as_mut().unwrap();
        assert_eq!(file.payload.as_ref().unwrap().len(), dummy_data.len());
        // Replace the data with the dummy data.
        file.payload = Some(dummy_data);
        assert_eq!(actual, expected);

        // Serialize back to bytes and compare everything up to the payload.
        let actual_bytes = expected.zvt_serialize();
        assert_eq!(actual_bytes[..26], bytes[..26]);
    }
}
