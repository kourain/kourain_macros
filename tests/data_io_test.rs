use kourain_macro::DataIO;
use std::io::Read;
// Minimal BinaryIO/stream stubs so the generated DataIO impl compiles in tests
#[derive(Debug, DataIO, Default)]
struct NguoiDung {
    _0_ten_: String,
    _1_tuoi_: u32,
    _2_test_4: [u8; 4],
    // _3_test_4: [u8; 8],
    _4_test_0x4: Vec<String>,
    _5_test: Vec<u8>, // reading until end of stream
    is_big_endian: bool,
}

#[test]
fn data_io_test() {
    let mut nguoi1 = NguoiDung::default();
    let byteinput: Vec<u8> = vec![
        // _0_ten_: String "AA"
        0x02, 0x00, 0x00, 0x00, b'A', b'A',
        // _1_tuoi_: u32 = 255
        0xFF, 0x00, 0x00, 0x00,
        // _2_test_4: [u8; 4]
        0x00, 0x00, 0x00, 0x2A,
        // _4_test_0x4: Vec<String> of 4 strings
        0x02, 0x00, 0x00, 0x00, b'H', b'i',   // "Hi"
        0x02, 0x00, 0x00, 0x00, b'A', b'B',   // "AB"
        0x02, 0x00, 0x00, 0x00, b'C', b'D',   // "CD"
        0x02, 0x00, 0x00, 0x00, b'E', b'F',   // "EF"
        // _5_test: Vec<u8> (read until end)
        0x01, 0x02, 0x03,
    ];
    let mut _reader = ByteReader::from_bytes(&byteinput);
    _reader.set_endian(bytebuffer::Endian::LittleEndian); // Set endianness for testing
    // Macro should generate these setters (note: field names start with `_`, so setter names get an extra `_`)
    let str = _reader.read_string().unwrap();
    let int = _reader.read_i32().unwrap();
    _reader.reset_cursors();
    std::println!("str: {}", str);
    std::println!("int: {}", int);
    nguoi1.read(&mut _reader);
    std::println!("nguoi1: {:?}", nguoi1);
    assert_eq!(nguoi1._0_ten_, "AA");
    assert_eq!(nguoi1._1_tuoi_, 255);
    assert_eq!(nguoi1._2_test_4, [0, 0, 0, 42]);
    assert_eq!(nguoi1._4_test_0x4, vec!["Hi", "AB", "CD", "EF"]);
    assert_eq!(nguoi1._5_test, vec![1, 2, 3]);
}
