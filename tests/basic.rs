/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate dtoa;
extern crate dtoa_short;
extern crate float_cmp;

use dtoa_short::{Floating, Notation};
use float_cmp::ApproxEqUlps;
use std::ops::Range;

struct SimpleNumberGenerator {
    whole_part: Range<i32>,
    digit1: Range<u8>,
    digit2: Range<u8>,
}

impl SimpleNumberGenerator {
    fn new(start: i32, end: i32) -> Self {
        SimpleNumberGenerator {
            whole_part: Range { start, end },
            digit1: Range { start: 0, end: 0 },
            digit2: Range { start: 0, end: 0 },
        }
    }
}

impl Iterator for SimpleNumberGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.digit2.next().is_some() {
            Some(format!("{}.{}{}", self.whole_part.start - 1,
                         self.digit1.start, self.digit2.start))
        } else {
            self.digit2 = Range { start: 0, end: 9 };
            if self.digit1.next().is_some() {
                Some(format!("{}.{}", self.whole_part.start - 1,
                             self.digit1.start))
            } else {
                self.digit1 = Range { start: 0, end: 9 };
                if self.whole_part.next().is_some() {
                    Some(format!("{}", self.whole_part.start - 1))
                } else {
                    None
                }
            }
        }
    }
}

#[test]
fn generator_correctness() {
    let mut last = 0;
    let end = 101;
    for (i, s) in SimpleNumberGenerator::new(0, end).enumerate() {
        let expected = i as f32 * 0.01;
        let result = s.parse::<f32>().unwrap();
        assert!(result.approx_eq_ulps(&expected, 1),
                "str = {}, result = {}, expected = {}", s, result, expected);
        last = i as i32;
    }
    assert_eq!(last, end * 100 - 1);
}

fn assert_expected_serialization<T: Floating>(value: T, expected: &str) {
    let mut result = String::new();
    let notation = dtoa_short::write(&mut result, value).unwrap();
    assert_eq!(result, expected);

    let exp_pos = expected.find('e');
    let has_dot = exp_pos.map(|exp_pos| &expected[..exp_pos])
                         .unwrap_or(expected)
                         .contains('.');
    let expected_notation = Notation {
        decimal_point: has_dot,
        scientific: exp_pos.is_some(),
    };
    assert_eq!(notation, expected_notation);
}

#[test]
fn roundtrip_simple_numbers_f32() {
    for s in SimpleNumberGenerator::new(0, 101) {
        let value = s.parse::<f32>().unwrap();
        assert_expected_serialization(value, &s);
    }
}

fn test_simple_number_percentage_f32<F: FnMut(f32, &str)>(mut test: F) {
    for s in SimpleNumberGenerator::new(0, 101) {
        let value = s.parse::<f32>().unwrap();
        let value = value / 100.;
        let value = value * 100.;
        test(value, &s);
    }
}

// This test is for checking that test_simple_number_percentage_f32
// actually works as expected, that it can generate numbers which dtoa
// fails to roundtrip.
#[test]
fn roundtrip_simple_numbers_percentage_dtoa_f32() {
    let mut count = 0;
    test_simple_number_percentage_f32(|value, expected| {
        let mut buf = dtoa::Buffer::new();
        let result = String::from(buf.format(value));
        if result != expected {
            count += 1;
        }
    });
    // This number may change if dtoa changes the algorithm.
    assert_eq!(count, 1135);
}

#[test]
fn roundtrip_simple_numbers_percentage_f32() {
    test_simple_number_percentage_f32(assert_expected_serialization);
}

#[test]
fn exponent_part_handling() {
    assert_expected_serialization(3.999999e30_f32, "4e30");
    assert_expected_serialization(-3.999999e30_f32, "-4e30");
    assert_expected_serialization(3.999999e-30_f32, "4e-30");
    assert_expected_serialization(-3.999999e-30_f32, "-4e-30");
    // 10e30 should really be 1e31, and 10e-30 should be 1e-29.
    // We can probably improve it later.
    assert_expected_serialization(9.999999e30_f32, "10e30");
    assert_expected_serialization(-9.999999e30_f32, "-10e30");
    assert_expected_serialization(9.999999e-30_f32, "10e-30");
    assert_expected_serialization(-9.999999e-30_f32, "-10e-30");
    // Regression test for assertion failure (https://bugzilla.mozilla.org/show_bug.cgi?id=1402419)
    assert_expected_serialization(-8192e17_f32, "-819200000000000000000");
}
