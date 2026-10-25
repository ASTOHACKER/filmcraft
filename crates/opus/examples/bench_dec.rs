//! Throwaway decoder benchmark (untracked; do not commit).
//! bench_dec <rate> <reps> <out.f32|-> <file.opus|file.pkt>
use filmcraft_opus::{Packet, StreamDecoder};

fn ogg_packets(data: &[u8]) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    let mut cur = Vec::new();
    let mut pos = 0;
    while pos + 27 <= data.len() && &data[pos..pos + 4] == b"OggS" {
        let nseg = data[pos + 26] as usize;
        let table = &data[pos + 27..pos + 27 + nseg];
        let mut body = pos + 27 + nseg;
        for &l in table {
            cur.extend_from_slice(&data[body..body + l as usize]);
            body += l as usize;
            if l < 255 {
                packets.push(std::mem::take(&mut cur));
            }
        }
        pos = body;
    }
    packets
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rate: u32 = a[1].parse().unwrap();
    let reps: usize = a[2].parse().unwrap();
    let out_path = &a[3];
    let path = &a[4];
    let data = std::fs::read(path).unwrap();
    let (ch, packets) = if path.ends_with(".opus") {
        let p = ogg_packets(&data);
        (p[0][9] as usize, p[2..].to_vec())
    } else {
        let ch = data[0] as usize;
        let mut pos = 3;
        let mut v = Vec::new();
        while pos + 2 <= data.len() {
            let l = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            v.push(data[pos..pos + l].to_vec());
            pos += l;
        }
        (ch, v)
    };
    let mut modes = [0usize; 3];
    for p in &packets {
        if let Ok(pk) = Packet::parse(p) {
            modes[pk.toc.mode as usize] += 1;
        }
    }
    let t0 = std::time::Instant::now();
    let mut out = Vec::with_capacity(packets.len() * 960 * ch);
    for _ in 0..reps {
        out.clear();
        let mut d = StreamDecoder::new(rate, ch).unwrap();
        for p in &packets {
            d.decode_interleaved(Some(p), &mut out).unwrap();
        }
    }
    let dt = t0.elapsed().as_secs_f64() / reps as f64;
    let secs = out.len() as f64 / ch as f64 / rate as f64;
    eprintln!("{path}: ch={ch} modes={modes:?} {:.4}s per pass for {:.1}s audio ({:.0}x realtime)", dt, secs, secs / dt);
    if out_path != "-" {
        let bytes: Vec<u8> = out.iter().flat_map(|v| v.to_le_bytes()).collect();
        std::fs::write(out_path, bytes).unwrap();
    }
}
