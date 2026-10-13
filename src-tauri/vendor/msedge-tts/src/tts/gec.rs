//! Sec-MS-GEC v2 token (`<16 hex>-<64 hex>`), required by the Read Aloud
//! endpoint since 2026-10-10. Ported from edge-tts 7.3.0 `drm.py`
//! (LGPL-3.0, https://github.com/rany2/edge-tts/issues/490).
//!
//! The SHA-256 half hashes a hidden 57-byte prefix plus the input; its first
//! block is precomputed into the constants below, so stock `sha2` can't
//! reproduce it. All arithmetic is mod 2^32.

const SHA256_INITIAL: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];
const SHA256_STATE: [u32; 8] = [
    0xFD9FBEA2, 0x91243CE7, 0xBE499094, 0xCD2E1E0A, 0x6C006F49, 0x04BD35AB, 0xFEB0A7E4, 0x03EFFBD2,
];
const SHA256_SCHEDULE: [u32; 14] = [
    0xEB24BF9E, 0x1558691F, 0xA0C09EE2, 0x0CFA32BA, 0xA622EE8C, 0xD1DD9491, 0xC0DD25A0, 0xF9E70AB6,
    0xF0C31573, 0x7058CE4E, 0x7188E606, 0x00E5AB33, 0xD258B060, 0x72766564,
];
const K: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];
const SHA256_ADJUSTMENT: u32 = 0xC9DC06A7;
const SHA256_SIGMA0_ADJUSTMENT: u32 = 0x059C0B80;
const SHA256_SIGMA1_ADJUSTMENT: u32 = 0x000B92C0;

const CS64_COEFFICIENTS: [(u32, u32); 14] = [
    (0xE82E91B1, 0xB32700D0),
    (0x44DE24C6, 0x88DFF51D),
    (0x3BEB65BC, 0x29514038),
    (0xC89C5933, 0xB328CBDF),
    (0x8D1EC211, 0xD405B4D5),
    (0x314EF5D5, 0x26E55F3C),
    (0xE92491B1, 0x607801C6),
    (0xF0A5B9D5, 0x82B2C1FB),
    (0x06064CF3, 0xFA51A3DA),
    (0xDA79FA60, 0x207CC464),
    (0xBED20560, 0x3C3C74F3),
    (0x89F9C9F8, 0xD2D14B61),
    (0x35FA38C6, 0xBB04A2C1),
    (0xCB49E3C6, 0xC5626167),
];
const CS64_FACTOR: u32 = 0x82893933;
const CS64_ADJUSTMENT0: u32 = 0xF2F90B1A;
const CS64_ADJUSTMENT1: u32 = 0x0B1A0000;

/// Token for `input` = `"{ticks}{TrustedClientToken}"`.
pub(super) fn token(input: &[u8]) -> String {
    let digest = sha256_digest(input);
    let (lo, hi) = cs64_digest(input, &digest);
    let mut out = String::with_capacity(81);
    for b in lo.to_le_bytes().iter().chain(&hi.to_le_bytes()) {
        out.push_str(&format!("{b:02X}"));
    }
    out.push('-');
    for b in digest {
        out.push_str(&format!("{b:02X}"));
    }
    out
}

fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

fn expand(w: &mut Vec<u32>, first: usize) {
    for i in first..64 {
        let v = sigma1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(sigma0(w[i - 15]))
            .wrapping_add(w[i - 16]);
        w.push(v);
    }
}

fn rounds(state: [u32; 8], w: &[u32], first: usize) -> [u32; 8] {
    let mut k = K;
    if first == 14 {
        k[14] = SHA256_ADJUSTMENT;
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
    for i in first..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = g ^ (e & (f ^ g));
        let t1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(k[i])
            .wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) | (c & (a | b));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(s0).wrapping_add(maj);
    }
    [a, b, c, d, e, f, g, h]
}

fn add(x: [u32; 8], y: [u32; 8]) -> [u32; 8] {
    std::array::from_fn(|i| x[i].wrapping_add(y[i]))
}

/// State after the precomputed first block, which ends with `d[..7]`.
fn sha256_init(d: &[u8]) -> [u32; 8] {
    let mut w = vec![0u32; 14];
    w.push(u32::from_be_bytes([0, d[0], d[1], d[2]]));
    w.push(u32::from_be_bytes([d[3], d[4], d[5], d[6]]));
    for i in 16..30 {
        let s1 = sigma1(w[i - 2]);
        let mut v = SHA256_SCHEDULE[i - 16].wrapping_add(if i == 16 {
            s1 ^ SHA256_SIGMA1_ADJUSTMENT
        } else {
            s1
        });
        if i >= 21 {
            v = v.wrapping_add(w[i - 7]);
        }
        if i == 29 {
            v = v.wrapping_add(sigma0(w[i - 15]) ^ SHA256_SIGMA0_ADJUSTMENT);
        }
        w.push(v);
    }
    w.push(
        sigma1(w[28])
            .wrapping_add(w[23])
            .wrapping_add(sigma0(w[15]))
            .wrapping_add(w[14])
            .wrapping_add(SHA256_ADJUSTMENT)
            .wrapping_sub(K[14]),
    );
    expand(&mut w, 31);
    add(SHA256_INITIAL, rounds(SHA256_STATE, &w, 14))
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    assert!(data.len() >= 7, "token input must be at least seven bytes");
    let length = 57 + data.len();
    let mut padded = data[7..].to_vec();
    padded.push(0x80);
    padded.resize(padded.len() + (55 - length as i64).rem_euclid(64) as usize, 0);
    padded.extend_from_slice(&(length as u64 * 8).to_be_bytes());
    let mut state = sha256_init(&data[..7]);
    for block in padded.chunks(64) {
        let mut w: Vec<u32> = block
            .chunks(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        expand(&mut w, 16);
        state = add(state, rounds(state, &w, 0));
    }
    let mut out = [0u8; 32];
    for (i, s) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&s.to_be_bytes());
    }
    out
}

fn cs64_digest(data: &[u8], digest: &[u8; 32]) -> (u32, u32) {
    let key_a = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]) | 1;
    let key_b = u32::from_le_bytes([digest[4], digest[5], digest[6], digest[7]]) | 1;
    let factors = [key_a.wrapping_mul(CS64_FACTOR), key_b.wrapping_mul(CS64_FACTOR)];
    let mut terms: Vec<u32> = CS64_COEFFICIENTS
        .iter()
        .enumerate()
        .map(|(i, (c, _))| c.wrapping_mul(factors[i % 2]))
        .collect();
    let mut offsets: Vec<u32> = CS64_COEFFICIENTS.iter().map(|(_, o)| *o).collect();

    let mut bytes = vec![0u8];
    bytes.extend_from_slice(data);
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    let mut words: Vec<u32> = bytes
        .chunks(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if words.len() % 2 == 1 {
        words.push(0);
    }
    for (i, &w) in words.iter().enumerate() {
        let even = i % 2 == 0;
        let mut term = w.wrapping_mul(if even { key_a } else { key_b });
        let mut offset = w
            .rotate_left(16)
            .wrapping_mul(if even { 0xEF0569FB } else { 0xC31713DB });
        if i == 0 {
            term = term.wrapping_add(CS64_ADJUSTMENT0.wrapping_mul(factors[0]));
            offset = offset.wrapping_add(CS64_ADJUSTMENT1);
        }
        terms.push(term);
        offsets.push(offset);
    }

    let (mut swap_acc, mut swap_last, mut state, mut acc) = (0u32, 0u32, 0u32, 0u32);
    for i in (0..terms.len()).step_by(2) {
        let (x, y) = (terms[i], terms[i + 1]);
        let q = offsets[i].wrapping_add(x).rotate_left(16).wrapping_mul(0x76F50F34);
        swap_acc = swap_acc.wrapping_add(q.rotate_left(16).wrapping_mul(0xC3EFEA97).wrapping_add(q));
        let q = offsets[i + 1].wrapping_add(y).rotate_left(16).wrapping_mul(0x1881CE3C);
        swap_last = q.rotate_left(16).wrapping_mul(0x35BD1EC9).wrapping_add(q);
        swap_acc = swap_acc.wrapping_add(swap_last);

        state = state.wrapping_mul(key_a).wrapping_add(x);
        state = state.rotate_left(16).wrapping_mul(0xCF98B111);
        state = state.rotate_left(16).wrapping_mul(0x87085B9F);
        state = state.rotate_left(16).wrapping_mul(0x12CEB96D);
        state = state.rotate_left(16).wrapping_mul(0x257E1D83);
        acc = acc.wrapping_add(state);
        let mut t = state.wrapping_mul(key_b).wrapping_add(y);
        t = t.rotate_left(16).wrapping_mul(0xA27416F5);
        t = t.rotate_left(16).wrapping_mul(0xD38396FF);
        state = t
            .rotate_left(16)
            .wrapping_mul(0x7C932B89)
            .rotate_left(16)
            .wrapping_mul(0xBFA49F69);
        acc = acc.wrapping_add(state);
    }
    (swap_last ^ state, swap_acc ^ acc)
}

#[cfg(test)]
mod tests {
    use super::token;

    // Golden values from edge-tts 7.3.0 `generate_sec_ms_gec_v2(unix_ts)`.
    #[test]
    fn matches_edge_tts_v2() {
        for (unix, want) in [
            (0u64, "52BC7A17B50DFEC9-28C4394775C8751CD2F6859C030C06DB1489ACAB3B3A7651831FC26895177ED6"),
            (1760000000, "6D50DDDB91812C33-DD68EFB01898B0750C9797D51A5FF93471DFDAE6888A1039D532CBDCCE54545D"),
            (1791900000, "5F546AEBA6D1BF75-CB47F061F310028902E5F18F2301717FD3A7439129E80730643BC73362B7134F"),
        ] {
            let ticks = (unix + 11644473600) / 300 * 300 * 10_000_000;
            let input = format!("{ticks}6A5AA1D4EAFF4E9FB37E23D68491D6F4");
            assert_eq!(token(input.as_bytes()), want, "unix {unix}");
        }
    }
}
