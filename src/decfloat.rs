// MIT License
//
// Copyright (c) 2021 Hajime Nakagami<nakagami@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use rust_decimal;
use super::utils::*;

use super::error::ValueError;

fn dpd_bit_to_int(dpd: u16, mask: u16) -> u16 {
    if (dpd & mask) != 0 {
        1
    } else {
        0
    }
}

fn dpd_to_int(dpd: u16) -> Result<u16, ValueError> {
    // Convert DPD encodined value to int (0-999)
    // dpd: DPD encoded value. 10bit unsigned int

    let mut b: [u16; 10] = Default::default();

    b[9] = dpd_bit_to_int(dpd, 0x0200);
    b[8] = dpd_bit_to_int(dpd, 0x0100);
    b[7] = dpd_bit_to_int(dpd, 0x0080);
    b[6] = dpd_bit_to_int(dpd, 0x0040);
    b[5] = dpd_bit_to_int(dpd, 0x0020);
    b[4] = dpd_bit_to_int(dpd, 0x0010);
    b[3] = dpd_bit_to_int(dpd, 0x0008);
    b[2] = dpd_bit_to_int(dpd, 0x0004);
    b[1] = dpd_bit_to_int(dpd, 0x0002);
    b[0] = dpd_bit_to_int(dpd, 0x0001);

    let mut d: [u16; 3] = Default::default();

    if b[3] == 0 {
        d[2] = b[9] * 4 + b[8] * 2 + b[7];
        d[1] = b[6] * 4 + b[5] * 2 + b[4];
        d[0] = b[2] * 4 + b[1] * 2 + b[0];
    } else if b[3] == 1 && b[2] == 0 && b[1] == 0 {
        d[2] = b[9] * 4 + b[8] * 2 + b[7];
        d[1] = b[6] * 4 + b[5] * 2 + b[4];
        d[0] = 8 + b[0];
    } else if b[3] == 1 && b[2] == 0 && b[1] == 1 {
        d[2] = b[9] * 4 + b[8] * 2 + b[7];
        d[1] = 8 + b[4];
        d[0] = b[6] * 4 + b[5] * 2 + b[0];
    } else if b[3] == 1 && b[2] == 1 && b[1] == 0 {
        d[2] = 8 + b[7];
        d[1] = b[6] * 4 + b[5] * 2 + b[4];
        d[0] = b[9] * 4 + b[8] * 2 + b[0];
    } else if b[6] == 0 && b[5] == 0 && b[3] == 1 && b[2] == 1 && b[1] == 1 {
        d[2] = 8 + b[7];
        d[1] = 8 + b[4];
        d[0] = b[9] * 4 + b[8] * 2 + b[0];
    } else if b[6] == 0 && b[5] == 1 && b[3] == 1 && b[2] == 1 && b[1] == 1 {
        d[2] = 8 + b[7];
        d[1] = b[9] * 4 + b[8] * 2 + b[4];
        d[0] = 8 + b[0];
    } else if b[6] == 1 && b[5] == 0 && b[3] == 1 && b[2] == 1 && b[1] == 1 {
        d[2] = b[9] * 4 + b[8] * 2 + b[7];
        d[1] = 8 + b[4];
        d[0] = 8 + b[0];
    } else if b[6] == 1 && b[5] == 1 && b[3] == 1 && b[2] == 1 && b[1] == 1 {
        d[2] = 8 + b[7];
        d[1] = 8 + b[4];
        d[0] = 8 + b[0];
    } else {
        return Err(ValueError::new("can't decode decimal"));
    }

    Ok(d[2] * 100 + d[1] * 10 + d[0])
}

fn calc_significand(prefix: i64, dpd_bits_arg: u128, num_bits: i32) -> Result<u128, ValueError> {
    // prefix: High bits integer value
    // dpd_bits: dpd encoded bits
    // num_bits: bit length of dpd_bits
    // https://en.wikipedia.org/wiki/Decimal128_floating-point_format#Densely_packed_decimal_significand_field
    let mut dpd_bits = dpd_bits_arg;
    let num_segments = num_bits / 10;
    let mut segments: Vec<u16> = Vec::new();
    for i in 0..num_segments {
        segments.push(dpd_bits as u16 & 0b1111111111);
        dpd_bits = dpd_bits >> 10;
    }
    segments.reverse();

    let mut v = prefix as u128;

    for dpd in segments {
        v = v * 1000 + dpd_to_int(dpd)? as u128;
    }

    Ok(v)
}

fn decimal128_to_sign_digits_exponent(b:Vec<u8>) -> Result<(i32, u128, i32), ValueError> {
    // https://en.wikipedia.org/wiki/Decimal128_floating-point_format
    let mut sign:i32 = 0;
    let mut exponent:i32 = 0;
    let mut prefix:i64 = 0;

    if (b[0] & 0x80) == 0x80 {
        sign = 1;
    }
    let cf = (((b[0]&0x7f) as u32) << 10) + ((b[1] as u32) <<2) + (b[2]>>6) as u32;
    if (cf & 0x1F000) == 0x1F000 {
        if sign == 1 {
            return Err(ValueError::new("NaN"));
        } else {
            return Err(ValueError::new("NaN"));
        }
    } else if (cf & 0x1F000) == 0x1E000 {
        if sign == 1 {
            return Err(ValueError::new("-Inf"));
        } else {
            return Err(ValueError::new("Inf"));
        }
    } else if (cf & 0x18000) == 0x00000 {
        exponent = (0x0000 + (cf & 0x00fff)) as i32;
        prefix = ((cf >> 12) & 0x07) as i64;
    } else if (cf & 0x18000) == 0x08000 {
        exponent = (0x1000 + (cf & 0x00fff)) as i32;
        prefix = ((cf >> 12) & 0x07) as i64;
    } else if (cf & 0x18000) == 0x10000 {
        exponent = (0x2000 + (cf & 0x00fff)) as i32;
        prefix = ((cf >> 12) & 0x07) as i64;
    } else if (cf & 0x1e000) == 0x18000 {
        exponent = (0x0000 + (cf & 0x00fff)) as i32;
        prefix = (8 + (cf>>12)&0x01) as i64;
    } else if (cf & 0x1e000) == 0x1a000 {
        exponent = (0x1000 + (cf & 0x00fff)) as i32;
        prefix = (8 + (cf>>12)&0x01) as i64;
    } else if (cf & 0x1e000) == 0x1c000 {
        exponent = (0x2000 + (cf & 0x00fff)) as i32;
        prefix = (8 + (cf>>12)&0x01) as i64;
    } else {
        return Err(ValueError::new("decimal128 value error"));
    }
    exponent -= 6176;

    let dpd_bits = bytes_to_uint128(&b);
    let mask:u128 = 0x3fffffffffffffffffffffffffff;
    let digits = calc_significand(prefix, dpd_bits & mask , 110)?;

    Ok((sign, digits, exponent))
}

/*

func decimalFixedToDecimal(b []byte, scale int32) decimal.Decimal {
    v, sign, digits, _ := decimal128ToSignDigitsExponent(b)
    if v != nil {
        return *v
    }
    if sign != 0 {
        digits.Mul(digits, big.NewInt(-1))
    }
    return decimal.NewFromBigInt(digits, scale)
}

func decimal64ToDecimal(b []byte) decimal.Decimal {
    // https://en.wikipedia.org/wiki/Decimal64_floating-point_format
    var prefix int64
    var sign int
    if (b[0] & 0x80) == 0x80 {
        sign = 1
    }
    cf := (uint32(b[0]) >> 2) & 0x1f
    exponent := ((int32(b[0]) & 3) << 6) + ((int32(b[1]) >> 2) & 0x3f)

    dpdBits := bytesToBigInt(b)
    mask := bigIntFromHexString("3ffffffffffff")
    dpdBits.And(dpdBits, mask)

    if cf == 0x1f {
        if sign == 1 {
            // Is there -NaN ?
            return decimal.NewFromFloat(math.NaN())
        }
        return decimal.NewFromFloat(math.NaN())
    } else if cf == 0x1e {
        if sign == 1 {
            return decimal.NewFromFloat(math.Inf(-1))
        }
        return decimal.NewFromFloat(math.Inf(1))
    } else if (cf & 0x18) == 0x00 {
        exponent = 0x000 + exponent
        prefix = int64(cf & 0x07)
    } else if (cf & 0x18) == 0x08 {
        exponent = 0x100 + exponent
        prefix = int64(cf & 0x07)
    } else if (cf & 0x18) == 0x10 {
        exponent = 0x200 + exponent
        prefix = int64(cf & 0x07)
    } else if (cf & 0x1e) == 0x18 {
        exponent = 0x000 + exponent
        prefix = int64(8 + cf&1)
    } else if (cf & 0x1e) == 0x1a {
        exponent = 0x100 + exponent
        prefix = int64(8 + cf&1)
    } else if (cf & 0x1e) == 0x1c {
        exponent = 0x200 + exponent
        prefix = int64(8 + cf&1)
    } else {
        panic("decimal64 value error")
    }
    digits := calcSignificand(prefix, dpdBits, 50)
    exponent -= 398

    if sign != 0 {
        digits.Mul(digits, big.NewInt(-1))
    }
    return decimal.NewFromBigInt(digits, exponent)
}

func decimal128ToDecimal(b []byte) decimal.Decimal {
    // https://en.wikipedia.org/wiki/Decimal64_floating-point_format
    v, sign, digits, exponent := decimal128ToSignDigitsExponent(b)
    if v != nil {
        return *v
    }
    if sign != 0 {
        digits.Mul(digits, big.NewInt(-1))
    }
    return decimal.NewFromBigInt(digits, exponent)
}
*/
