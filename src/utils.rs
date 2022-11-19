pub trait ChangeMinMax {
    fn change_min(&mut self, v: Self) -> bool;
    fn change_max(&mut self, v: Self) -> bool;
}

impl<T: PartialOrd> ChangeMinMax for T {
    fn change_min(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }

    fn change_max(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

pub fn decode_num_to_u64(data: &[u8]) -> u64 {
    let mut value = 0;

    for &d in data {
        value *= 10;
        value += d as u64 - 48;
    }

    value
}

#[allow(dead_code)]
pub fn decode_base64_to_f64(data: &[u8]) -> Vec<f64> {
    const BASE64_MAP: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut stream = vec![];

    let mut cursor = 0;

    while cursor + 4 <= data.len() {
        let mut buffer = 0u32;

        for i in 0..4 {
            let c = data[cursor + i];
            let shift = 6 * (3 - i);

            for (i, &d) in BASE64_MAP.iter().enumerate() {
                if c == d {
                    buffer |= (i as u32) << shift;
                }
            }
        }

        for i in 0..3 {
            let shift = 8 * (2 - i);
            let value = (buffer >> shift) as u8;
            stream.push(value);
        }

        cursor += 4;
    }

    let mut result = vec![];
    cursor = 0;

    while cursor + 8 <= stream.len() {
        let p = stream.as_ptr() as *const f64;
        let x = unsafe { *p.offset(cursor as isize / 8) };
        result.push(x);
        cursor += 8;
    }

    result
}
