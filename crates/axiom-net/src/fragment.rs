//! UDP datagram fragmentation and reassembly (MTU-aware).

pub const DEFAULT_MTU: usize = 1200;

pub fn fragment(payload: &[u8], mtu: usize) -> Vec<Vec<u8>> {
    let header_reserve = 16;
    let chunk = mtu.saturating_sub(header_reserve).max(1);
    if payload.len() <= chunk {
        return vec![payload.to_vec()];
    }
    payload.chunks(chunk).map(|c| c.to_vec()).collect()
}

pub fn reassemble(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for c in chunks {
        out.extend_from_slice(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_reassemble() {
        let data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        let frags = fragment(&data, 512);
        assert!(frags.len() > 1);
        let joined = reassemble(&frags);
        assert_eq!(joined, data);
    }
}
