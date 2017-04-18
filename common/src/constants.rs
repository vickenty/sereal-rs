pub const MAGIC_V1: u32 = 0x6C72733D;
pub const MAGIC_V3: u32 = 0x6C72F33D;

pub const PROTO_V2: u8 = 2;
pub const PROTO_V3: u8 = 3;
pub const PROTO_V4: u8 = 4;

pub const TYPE_RAW: u8 = 0;
pub const TYPE_SNAPPY: u8 = 2;
pub const TYPE_ZLIB: u8 = 3;
pub const TYPE_ZSTD: u8 = 4;

pub const OPT_USER_METADATA: u8 = 1;

pub const POS_0: u8 = 0x00;
pub const POS_1: u8 = 0x01;
pub const POS_2: u8 = 0x02;
pub const POS_3: u8 = 0x03;
pub const POS_4: u8 = 0x04;
pub const POS_5: u8 = 0x05;
pub const POS_6: u8 = 0x06;
pub const POS_7: u8 = 0x07;
pub const POS_8: u8 = 0x08;
pub const POS_9: u8 = 0x09;
pub const POS_10: u8 = 0x0a;
pub const POS_11: u8 = 0x0b;
pub const POS_12: u8 = 0x0c;
pub const POS_13: u8 = 0x0d;
pub const POS_14: u8 = 0x0e;
pub const POS_15: u8 = 0x0f;
pub const NEG_16: u8 = 0x10;
pub const NEG_15: u8 = 0x11;
pub const NEG_14: u8 = 0x12;
pub const NEG_13: u8 = 0x13;
pub const NEG_12: u8 = 0x14;
pub const NEG_11: u8 = 0x15;
pub const NEG_10: u8 = 0x16;
pub const NEG_9: u8 = 0x17;
pub const NEG_8: u8 = 0x18;
pub const NEG_7: u8 = 0x19;
pub const NEG_6: u8 = 0x1a;
pub const NEG_5: u8 = 0x1b;
pub const NEG_4: u8 = 0x1c;
pub const NEG_3: u8 = 0x1d;
pub const NEG_2: u8 = 0x1e;
pub const NEG_1: u8 = 0x1f;
pub const VARINT: u8 = 0x20;
pub const ZIGZAG: u8 = 0x21;
pub const FLOAT: u8 = 0x22;
pub const DOUBLE: u8 = 0x23;
pub const LONG_DOUBLE: u8 = 0x24;
pub const UNDEF: u8 = 0x25;
pub const BINARY: u8 = 0x26;
pub const STR_UTF8: u8 = 0x27;
pub const REFN: u8 = 0x28;
pub const REFP: u8 = 0x29;
pub const HASH: u8 = 0x2a;
pub const ARRAY: u8 = 0x2b;
pub const OBJECT: u8 = 0x2c;
pub const OBJECTV: u8 = 0x2d;
pub const ALIAS: u8 = 0x2e;
pub const COPY: u8 = 0x2f;
pub const WEAKEN: u8 = 0x30;
pub const REGEXP: u8 = 0x31;
pub const OBJECT_FREEZE: u8 = 0x32;
pub const OBJECTV_FREEZE: u8 = 0x33;
pub const RESERVED_0: u8 = 0x34;
pub const RESERVED_1: u8 = 0x35;
pub const RESERVED_2: u8 = 0x36;
pub const RESERVED_3: u8 = 0x37;
pub const RESERVED_4: u8 = 0x38;
pub const CANONICAL_UNDEF: u8 = 0x39;
pub const FALSE: u8 = 0x3a;
pub const TRUE: u8 = 0x3b;
pub const MANY: u8 = 0x3c;
pub const PACKET_START: u8 = 0x3d;
pub const EXTEND: u8 = 0x3e;
pub const PAD: u8 = 0x3f;
pub const ARRAYREF_0: u8 = 0x40;
pub const ARRAYREF_1: u8 = 0x41;
pub const ARRAYREF_2: u8 = 0x42;
pub const ARRAYREF_3: u8 = 0x43;
pub const ARRAYREF_4: u8 = 0x44;
pub const ARRAYREF_5: u8 = 0x45;
pub const ARRAYREF_6: u8 = 0x46;
pub const ARRAYREF_7: u8 = 0x47;
pub const ARRAYREF_8: u8 = 0x48;
pub const ARRAYREF_9: u8 = 0x49;
pub const ARRAYREF_10: u8 = 0x4a;
pub const ARRAYREF_11: u8 = 0x4b;
pub const ARRAYREF_12: u8 = 0x4c;
pub const ARRAYREF_13: u8 = 0x4d;
pub const ARRAYREF_14: u8 = 0x4e;
pub const ARRAYREF_15: u8 = 0x4f;
pub const HASHREF_0: u8 = 0x50;
pub const HASHREF_1: u8 = 0x51;
pub const HASHREF_2: u8 = 0x52;
pub const HASHREF_3: u8 = 0x53;
pub const HASHREF_4: u8 = 0x54;
pub const HASHREF_5: u8 = 0x55;
pub const HASHREF_6: u8 = 0x56;
pub const HASHREF_7: u8 = 0x57;
pub const HASHREF_8: u8 = 0x58;
pub const HASHREF_9: u8 = 0x59;
pub const HASHREF_10: u8 = 0x5a;
pub const HASHREF_11: u8 = 0x5b;
pub const HASHREF_12: u8 = 0x5c;
pub const HASHREF_13: u8 = 0x5d;
pub const HASHREF_14: u8 = 0x5e;
pub const HASHREF_15: u8 = 0x5f;
pub const SHORT_BINARY_0: u8 = 0x60;
pub const SHORT_BINARY_1: u8 = 0x61;
pub const SHORT_BINARY_2: u8 = 0x62;
pub const SHORT_BINARY_3: u8 = 0x63;
pub const SHORT_BINARY_4: u8 = 0x64;
pub const SHORT_BINARY_5: u8 = 0x65;
pub const SHORT_BINARY_6: u8 = 0x66;
pub const SHORT_BINARY_7: u8 = 0x67;
pub const SHORT_BINARY_8: u8 = 0x68;
pub const SHORT_BINARY_9: u8 = 0x69;
pub const SHORT_BINARY_10: u8 = 0x6a;
pub const SHORT_BINARY_11: u8 = 0x6b;
pub const SHORT_BINARY_12: u8 = 0x6c;
pub const SHORT_BINARY_13: u8 = 0x6d;
pub const SHORT_BINARY_14: u8 = 0x6e;
pub const SHORT_BINARY_15: u8 = 0x6f;
pub const SHORT_BINARY_16: u8 = 0x70;
pub const SHORT_BINARY_17: u8 = 0x71;
pub const SHORT_BINARY_18: u8 = 0x72;
pub const SHORT_BINARY_19: u8 = 0x73;
pub const SHORT_BINARY_20: u8 = 0x74;
pub const SHORT_BINARY_21: u8 = 0x75;
pub const SHORT_BINARY_22: u8 = 0x76;
pub const SHORT_BINARY_23: u8 = 0x77;
pub const SHORT_BINARY_24: u8 = 0x78;
pub const SHORT_BINARY_25: u8 = 0x79;
pub const SHORT_BINARY_26: u8 = 0x7a;
pub const SHORT_BINARY_27: u8 = 0x7b;
pub const SHORT_BINARY_28: u8 = 0x7c;
pub const SHORT_BINARY_29: u8 = 0x7d;
pub const SHORT_BINARY_30: u8 = 0x7e;
pub const SHORT_BINARY_31: u8 = 0x7f;
pub const TRACK_BIT: u8 = 0x80;
pub const TYPE_MASK: u8 = 0x7f;