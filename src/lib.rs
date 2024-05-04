fn linecomp(l1: &str, l2: &str) -> Ordering {
    let mut i1 = l1.char_indices().peekable();
    let mut i2 = l2.char_indices().peekable();
    let mut l1_larger = Ordering::Greater;

    'next_field: loop {
        // handle negative prefixes and end of lines
        match (i1.peek(), i2.peek()) {
            (Some((_, _)), None) => return Ordering::Greater,  // end of line for l2, so l1 > l2
            (None, Some((_, _))) => return Ordering::Less,  // end of line for l1, so l1 < l2
            (None, None) => return Ordering::Equal,  // end of both lines, so l1 == l2
            (Some((_, '-')), Some((_, '-'))) => {  // both l1 and l2 have negative prefixes
                l1_larger = Ordering::Less;  // invert the comparison of absolute values
                // skip the negative prefixes and start comparing absolute values
                i1.next();
                i2.next();
            }
            (Some((_, '-')), Some((_, _))) => {  // only l1 has a negative prefix, so l1 < l2
                return Ordering::Less;
            }
            (Some((_, _)), Some((_, '-'))) => {  // only l2 has a negative prefix, so l1 > l2
                return Ordering::Greater;
            }
            (Some((_, _)), Some((_, _))) => {}  // neither has a negative prefix, continue
        }

        // skip leading zeros in i1
        while let Some((_, c)) = i1.peek() {
            if *c == '0' {
                i1.next();
            } else {
                break;
            }
        }

        // skip leading zeros in i2
        while let Some((_, c)) = i2.peek() {
            if *c == '0' {
                i2.next();
            } else {
                break;
            }
        }

        let mut comparison = Ordering::Equal;
        loop {
            match (i1.next(), i2.next()) {
                (Some((_, _)), None) => {  // end of line for l2
                    return comparison.then(l1_larger);  // so |l1| > |l2| unless digits differed
                },
                (None, Some((_, _))) => {  // end of line for l1
                    return match comparison {
                        Ordering::Less => l1_larger.reverse(),  // by digits |l1| < |l2| anyway
                        Ordering::Equal => l1_larger.reverse(),  // by convention, longer is larger
                        Ordering::Greater => l1_larger,  // but digits differed and |l1| > |l2|
                    };
                },
                (None, None) => {  // end of both lines, result depends on digit comparison
                    return match comparison {
                        Ordering::Less => l1_larger.reverse(),
                        Ordering::Equal => Ordering::Equal,
                        Ordering::Greater => l1_larger,
                    };
                }
                (Some((_, c1)), Some((_, c2))) => {
                    if !c1.is_ascii_digit() {
                        if c2.is_ascii_digit() {  // l1 has a non-digit character, l2 has a digit
                            return l1_larger.reverse();  // so l2 is longer, thus |l1| < |l2|
                        };
                        // both l1 and l2 have a non-digit character after the same number of digits
                        // so the result depends on digit comparisons before the non-digit character
                        match comparison {
                            // non-equal comparison before the non-digit character
                            Ordering::Less => return l1_larger.reverse(),
                            Ordering::Greater => return l1_larger,
                            Ordering::Equal => {
                                // l1 and l2 have the same digits until the non-digit character
                                if c1 == '.' {
                                    if c2 == '.' {
                                        // it was a decimal point in both, and as both have the same
                                        // integer part, now compare the fractional parts
                                        break;
                                    }
                                    return l1_larger;  // l1 has fraction, l2 not, so |l1| > |l2|
                                }
                                if c2 == '.' {
                                    return l1_larger.reverse();  // l2 has, l1 not, so |l1| < |l2|
                                }
                                // both l1 and l2 have a non-digit character, and it's not a decimal
                                // so we shift to non-numeric comparison below
                            }
                        }
                    } else if !c2.is_ascii_digit() {
                        return l1_larger;  // l2 has a non-digit, l1 a digit, so |l1| > |l2|
                    }
                    // compare the next characters in l1 and l2, digit or not
                    if comparison.is_eq() {  // all characters so far have been equal
                        comparison = c1.cmp(&c2);  // so compare the current characters
                        // note: we don't draw any conclusions yet, as we don't know if the number
                        // of digits is the same in both lines
                    }
                }
            }
        }

        // l1 and l2 have the same integer part, compare the fractional part
        loop {
            match (i1.next(), i2.next()) {
                (Some((_, _)), None) => return l1_larger,  // end of line for l2, so |l1| > |l2|
                (None, Some((_, _))) => return l1_larger.reverse(),  // EOL for l1, so |l1| < |l2|
                (None, None) => return Ordering::Equal,  // end of both lines, so l1 == l2
                (Some((_, c1)), Some((_, c2))) => {
                    if c1 == '\t' {  // field ends in l1
                        if c2 == '\t' {  // field ends in l2, too
                            // l1 and l2 have the same fractional part, they are equal
                            continue 'next_field;
                        }
                        // l1 has fewer fractional digits than l2, so |l1| < |l2|
                        return l1_larger.reverse();
                    }
                    if c2 == '\t' {  // field ends in l2
                        // l1 has more fractional digits than l2, so |l1| > |l2|
                        return l1_larger;
                    }
                    match c1.cmp(&c2) {
                        Ordering::Less => return l1_larger.reverse(),
                        Ordering::Greater => return l1_larger,
                        Ordering::Equal => continue,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate rstest;

use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    // integers, equal length
    #[case("123", "123", Ordering::Equal)]
    #[case("123", "124", Ordering::Less)]
    #[case("124", "123", Ordering::Greater)]
    // integers, different length
    #[case("123", "1234", Ordering::Less)]
    #[case("1234", "123", Ordering::Greater)]
    // negative integers
    #[case("-123", "123", Ordering::Less)]
    #[case("123", "-123", Ordering::Greater)]
    #[case("-123", "-123", Ordering::Equal)]
    #[case("-123", "-124", Ordering::Greater)]
    #[case("-124", "-123", Ordering::Less)]
    // integers vs. floats
    #[case("123", "122.9", Ordering::Greater)]
    #[case("123", "123.0", Ordering::Less)] // by convention, shorter notation is less than longer
    #[case("123", "123.1", Ordering::Less)]
    #[case("123.1", "123", Ordering::Greater)]
    #[case("123.0", "123", Ordering::Greater)] // by convention, shorter notation is less than longer
    #[case("122.9", "123", Ordering::Less)]
    // floats vs. floats, equal length, same integer part
    #[case("123.1", "123.1", Ordering::Equal)]
    #[case("123.0", "123.1", Ordering::Less)]
    #[case("123.1", "123.0", Ordering::Greater)]
    // floats vs. floats, different length, same integer part
    #[case("123.0", "123.01", Ordering::Less)]
    #[case("123.01", "123.0", Ordering::Greater)]
    #[case("123.0", "123.00", Ordering::Less)] // by convention, shorter notation less than longer
    #[case("123.00", "123.0", Ordering::Greater)] // by convention, shorter notation less than longer-
    // floats vs. floats, same length, different integer part
    #[case("123.0", "124.0", Ordering::Less)]
    #[case("124.0", "123.0", Ordering::Greater)]
    #[case("123.0", "124.1", Ordering::Less)]
    #[case("124.1", "123.0", Ordering::Greater)]
    // floats vs. floats, different length, different integer part
    #[case("123.0", "124.00", Ordering::Less)]
    #[case("124.00", "123.0", Ordering::Greater)]
    #[case("123.0", "124.01", Ordering::Less)]
    #[case("124.01", "123.0", Ordering::Greater)]
    // negative floats
    #[case("-123.0", "123.0", Ordering::Less)]
    #[case("123.0", "-123.0", Ordering::Greater)]
    #[case("-123.0", "-123.0", Ordering::Equal)]
    #[case("-123.0", "-123.1", Ordering::Greater)]
    #[case("-123.1", "-123.0", Ordering::Less)]
    #[case("-123.0", "-123.00", Ordering::Greater)] // by convention, shorter notation less than long
    #[case("-123.00", "-123.0", Ordering::Less)] // by convention, shorter notation less than long
    #[case("-123.02", "-123.01", Ordering::Less)]
    #[case("-123.01", "-123.02", Ordering::Greater)]
    #[case("-123.00", "-123.01", Ordering::Greater)]
    #[case("-123.01", "-123.00", Ordering::Less)]
    #[case("-123.0", "-123.01", Ordering::Greater)]
    #[case("-123.01", "-123.0", Ordering::Less)]
    #[case("-123.0", "-124.0", Ordering::Greater)]
    #[case("-124.0", "-123.0", Ordering::Less)]
    #[case("-123.0", "-124.1", Ordering::Greater)]
    #[case("-124.1", "-123.0", Ordering::Less)]
    #[case("-123.0", "-124.00", Ordering::Greater)]
    #[case("-124.00", "-123.0", Ordering::Less)]
    #[case("-123.0", "-124.01", Ordering::Greater)]
    #[case("-124.01", "-123.0", Ordering::Less)]
    // negative integers vs. floats
    #[case("-123", "123.0", Ordering::Less)]
    #[case("123", "-123.0", Ordering::Greater)]
    #[case("-123.0", "123", Ordering::Less)]
    #[case("123.0", "-123", Ordering::Greater)]
    #[case("-123", "123.1", Ordering::Less)]
    #[case("123.1", "-123", Ordering::Greater)]
    #[case("-123.1", "123", Ordering::Less)]
    #[case("123", "-123.1", Ordering::Greater)]
    #[case("-123", "123.0", Ordering::Less)]
    #[case("123", "-123.0", Ordering::Greater)]
    #[case("-123.0", "123", Ordering::Less)]
    #[case("123.0", "-123", Ordering::Greater)]
    #[case("-123", "123.00", Ordering::Less)]
    #[case("123.00", "-123", Ordering::Greater)]
    #[case("-123.00", "123", Ordering::Less)]
    #[case("123", "-123.00", Ordering::Greater)]
    #[case("-123", "123.01", Ordering::Less)]
    #[case("123.01", "-123", Ordering::Greater)]
    #[case("-123.01", "123", Ordering::Less)]
    #[case("123", "-123.01", Ordering::Greater)]
    fn test_linecomp(#[case] l1: &str, #[case] l2: &str, #[case] expected: Ordering) {
        assert_eq!(
            linecomp(l1, l2),
            expected,
            "linecomp({}, {}) == {}, expected {}",
            l1,
            l2,
            linecomp(l1, l2) as i8,
            expected as i8,
        );
    }
}
