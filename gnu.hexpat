import std.string;
import std.core;

enum Tag: u8 {
    True = 0,
    False = 1,
    Symbol = 2,
    Int32 = 3,
    Double = 4,
    Complex = 5,
    Null = 6,
    Pair = 7,
    Vector = 8,
    Dict = 9,
    UniformVector = 0xa,
    UInt64 = 0xb,
    Tuple = 0xc,
    Int64 = 0xd,    
};

using GnuRadioPMT;

struct GnuRadioPMT {
    u64 bytes = 0;
    Tag tag;
    
    if(tag == Tag::Dict || tag == Tag::Pair) {
        GnuRadioPMT car;
        GnuRadioPMT cdr;
        if(car.tag == Tag::Symbol) {
           if(std::core::has_member(cdr, "val")) {
                bytes = cdr.val;
           }
        }
    } else if (tag == Tag::Symbol) {
        std::string::SizedString<be u16> symbol;
    } else if (tag == Tag::UInt64) {
        be u64 val;
    } else if (tag == Tag::Int32) {
        be s32 val;
    } else if (tag == Tag::Tuple) {
        be u32 tupleSize;
        GnuRadioPMT contents[tupleSize];
    } else if (tag == Tag::Double) {
        be double val;
    }
};

struct GnuRadioHeader {
    GnuRadioPMT base;
    GnuRadioPMT meta;

    if(base.bytes > 0) {
        u8 guamedo;
    }
};

struct GnuRadioFile {
    GnuRadioHeader first;
};

GnuRadioFile file @ 0x0;