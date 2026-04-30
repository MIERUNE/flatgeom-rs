//! postcard ± lz4 serialize/deserialize speed and size, comparing flatgeom
//! (flat layout) and geo_types (jagged Vec-of-Vec layout) for the WKT fixtures.
//!
//! Three coord layouts:
//!   * `f32` — half-precision float
//!   * `i16` — vector-tile-style quantized to [0, 4096]
//!   * `i16-d` (flat-tile) — same coords, custom delta + zigzag varint encoding
//!     (no leading 0 / trailing length sentinels in offsets)
//!
//! Run with:
//!   cargo run --release --example postcard_bench --features serde

use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use flatgeom::geozero::ToFlatgeom;
use flatgeom::{MultiPolygon, MultiPolygonRef, Polygon, PolygonRef};
use geozero::ToGeo;
use geozero::wkt::Wkt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const BENCH_SECS: f64 = 1.0;
const WARMUP_SECS: f64 = 0.1;

// ---- Custom tile-i16 wire format (streaming, no intermediate Vec<i32>) ----
//
// Polygon body wire layout (postcard):
//   nc:    u32                      coord count
//   nr:    u32                      ring count
//   inner: seq<u32>                 polygon-local inner ring offsets
//                                   (skips leading 0 and trailing nc)
//   zigs:  seq<i32>                 flat zigzag varint deltas; first vertex of
//                                   each ring is absolute, rest are deltas
//
// Serialize / Deserialize are written by hand so the zigs are streamed
// element-by-element through the (de)serializer without a Vec<i32> staging
// buffer.

use core::fmt;
use serde::de::{DeserializeSeed, Error as DeError, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};

struct TilePolygon(Polygon<[i16; 2]>);
struct TileMultiPolygon(MultiPolygon<[i16; 2]>);

const TILE_POLY_FIELDS: &[&str] = &["nc", "nr", "ring_lens", "zigs"];

fn serialize_polygon_struct<P, S>(p: &P, ser: S) -> Result<S::Ok, S::Error>
where
    P: PolygonRef<[i16; 2]>,
    S: serde::Serializer,
{
    let raw = p.raw_coords();
    let ring_offsets = p.ring_offsets();
    let nc = raw.len() as u32;
    let nr = ring_offsets.len().saturating_sub(1) as u32;

    let mut s = ser.serialize_struct("TilePolygon", 4)?;
    s.serialize_field("nc", &nc)?;
    s.serialize_field("nr", &nr)?;
    s.serialize_field("ring_lens", &RingLengthsSer(p.ring_lengths()))?;
    s.serialize_field("zigs", &ZigsSer { raw, ring_offsets })?;
    s.end()
}

/// Writes ring lengths (= deltas of `ring_offsets`). For N rings, writes N
/// values. Each entry is `ring_offsets[i+1] - ring_offsets[i]`. The leading
/// 0 sentinel is implicit (skipped), and the trailing total is implicit in
/// the sum of the deltas.
struct RingLengthsSer<I>(I);

impl<I> Serialize for RingLengthsSer<I>
where
    I: Clone + ExactSizeIterator<Item = u32>,
{
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let iter = self.0.clone();
        let mut seq = ser.serialize_seq(Some(iter.len()))?;
        for len in iter {
            seq.serialize_element(&len)?;
        }
        seq.end()
    }
}

struct ZigsSer<'a> {
    raw: &'a [[i16; 2]],
    ring_offsets: &'a [u32],
}

impl Serialize for ZigsSer<'_> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let total = 2 * self.raw.len();
        let mut seq = ser.serialize_seq(Some(total))?;
        let base = self.ring_offsets.first().copied().unwrap_or(0);
        let nr = self.ring_offsets.len().saturating_sub(1);
        for r in 0..nr {
            let s_idx = (self.ring_offsets[r] - base) as usize;
            let t_idx = (self.ring_offsets[r + 1] - base) as usize;
            let ring = &self.raw[s_idx..t_idx];
            if ring.is_empty() {
                continue;
            }
            seq.serialize_element(&(ring[0][0] as i32))?;
            seq.serialize_element(&(ring[0][1] as i32))?;
            let mut prev = ring[0];
            for c in &ring[1..] {
                seq.serialize_element(&(c[0] as i32 - prev[0] as i32))?;
                seq.serialize_element(&(c[1] as i32 - prev[1] as i32))?;
                prev = *c;
            }
        }
        seq.end()
    }
}

impl Serialize for TilePolygon {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        serialize_polygon_struct(&self.0, ser)
    }
}

// MultiPolygon wire format (MLT-aligned, single topology stream):
//   topo: Vec<u32>   = [P, r_0, .., r_{P-1}, v_0, v_1, .., v_{total_rings-1}]
//                       └─1─┘ └ ring counts ┘ └ vertex counts (= ring lengths) ─┘
//   zigs: Vec<i32>   = flat zigzag deltas (first vertex of each ring absolute)

struct TopoStreamSer<'a> {
    mp: &'a MultiPolygon<[i16; 2]>,
}

impl Serialize for TopoStreamSer<'_> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mp = self.mp;
        let p = mp.len();
        let polygon_ring_counts = mp.polygon_ring_counts();
        let ring_lengths = mp.ring_lengths();

        let total = 1 + polygon_ring_counts.len() + ring_lengths.len();
        let mut seq = ser.serialize_seq(Some(total))?;
        seq.serialize_element(&(p as u32))?;
        for count in polygon_ring_counts {
            seq.serialize_element(&count)?;
        }
        for len in ring_lengths {
            seq.serialize_element(&len)?;
        }
        seq.end()
    }
}

struct MpZigsSer<'a> {
    mp: &'a MultiPolygon<[i16; 2]>,
}

impl Serialize for MpZigsSer<'_> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mp = self.mp;
        let raw = mp.raw_coords();
        let ring_offsets = mp.ring_offsets();
        let total_rings = ring_offsets.len().saturating_sub(1);

        let mut seq = ser.serialize_seq(Some(2 * raw.len()))?;
        for r in 0..total_rings {
            let s_idx = ring_offsets[r] as usize;
            let t_idx = ring_offsets[r + 1] as usize;
            let ring = &raw[s_idx..t_idx];
            if ring.is_empty() {
                continue;
            }
            seq.serialize_element(&(ring[0][0] as i32))?;
            seq.serialize_element(&(ring[0][1] as i32))?;
            let mut prev = ring[0];
            for c in &ring[1..] {
                seq.serialize_element(&(c[0] as i32 - prev[0] as i32))?;
                seq.serialize_element(&(c[1] as i32 - prev[1] as i32))?;
                prev = *c;
            }
        }
        seq.end()
    }
}

impl Serialize for TileMultiPolygon {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_struct("TileMultiPolygon", 2)?;
        s.serialize_field("topo", &TopoStreamSer { mp: &self.0 })?;
        s.serialize_field("zigs", &MpZigsSer { mp: &self.0 })?;
        s.end()
    }
}

// ---- Deserialize (streaming) ----

struct ZigsSeed<'a> {
    nr: usize,
    ring_offsets: &'a [u32],
}

impl<'de, 'a> DeserializeSeed<'de> for ZigsSeed<'a> {
    type Value = Vec<[i16; 2]>;
    fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        de.deserialize_seq(ZigsVisitor {
            nr: self.nr,
            ring_offsets: self.ring_offsets,
        })
    }
}

struct ZigsVisitor<'a> {
    nr: usize,
    ring_offsets: &'a [u32],
}

impl<'de, 'a> Visitor<'de> for ZigsVisitor<'a> {
    type Value = Vec<[i16; 2]>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("zigzag delta sequence")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let total = *self.ring_offsets.last().unwrap_or(&0) as usize;
        let mut coords = Vec::with_capacity(total);
        for r in 0..self.nr {
            let len = (self.ring_offsets[r + 1] - self.ring_offsets[r]) as usize;
            if len == 0 {
                continue;
            }
            let x: i32 = seq
                .next_element()?
                .ok_or_else(|| A::Error::custom("zigs eof"))?;
            let y: i32 = seq
                .next_element()?
                .ok_or_else(|| A::Error::custom("zigs eof"))?;
            let first = [x as i16, y as i16];
            coords.push(first);
            let mut prev = first;
            for _ in 1..len {
                let dx: i32 = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("zigs eof"))?;
                let dy: i32 = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("zigs eof"))?;
                let c = [(prev[0] as i32 + dx) as i16, (prev[1] as i32 + dy) as i16];
                coords.push(c);
                prev = c;
            }
        }
        Ok(coords)
    }
}

struct TilePolygonVisitor;
impl<'de> Visitor<'de> for TilePolygonVisitor {
    type Value = TilePolygon;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("TilePolygon")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let nc: u32 = seq
            .next_element()?
            .ok_or_else(|| A::Error::custom("missing nc"))?;
        let nr: u32 = seq
            .next_element()?
            .ok_or_else(|| A::Error::custom("missing nr"))?;
        let ring_lens: Vec<u32> = seq
            .next_element()?
            .ok_or_else(|| A::Error::custom("missing ring_lens"))?;

        // Reconstruct cumulative ring_offsets from ring lengths.
        // Resulting length is N+1 (one entry per ring + leading 0 sentinel).
        let mut ring_offsets = Vec::with_capacity(nr as usize + 1);
        ring_offsets.push(0u32);
        let mut acc = 0u32;
        for d in ring_lens {
            acc += d;
            ring_offsets.push(acc);
        }

        let coords = seq
            .next_element_seed(ZigsSeed {
                nr: nr as usize,
                ring_offsets: &ring_offsets,
            })?
            .ok_or_else(|| A::Error::custom("missing zigs"))?;

        // Sanity: accumulated total should equal advertised coord count.
        debug_assert_eq!(acc, nc);

        Ok(TilePolygon(Polygon::from_raw_unchecked(
            coords,
            ring_offsets,
        )))
    }
}

impl<'de> Deserialize<'de> for TilePolygon {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_struct("TilePolygon", TILE_POLY_FIELDS, TilePolygonVisitor)
    }
}

impl<'de> Deserialize<'de> for TileMultiPolygon {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = TileMultiPolygon;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("TileMultiPolygon")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let topo: Vec<u32> = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("missing topo"))?;
                if topo.is_empty() {
                    return Ok(TileMultiPolygon(MultiPolygon::new()));
                }
                let p = topo[0] as usize;
                if topo.len() < 1 + p {
                    return Err(A::Error::custom("topo too short"));
                }
                let ring_counts = &topo[1..1 + p];
                let total_rings: usize = ring_counts.iter().map(|&c| c as usize).sum();
                if topo.len() != 1 + p + total_rings {
                    return Err(A::Error::custom("topo length mismatch"));
                }
                let vertex_counts = &topo[1 + p..];

                // Reconstruct cumulative offsets.
                let mut polygon_offsets = Vec::with_capacity(p + 1);
                polygon_offsets.push(0u32);
                let mut acc = 0u32;
                for &c in ring_counts {
                    acc += c;
                    polygon_offsets.push(acc);
                }

                let mut ring_offsets = Vec::with_capacity(total_rings + 1);
                ring_offsets.push(0u32);
                let mut acc = 0u32;
                for &v in vertex_counts {
                    acc += v;
                    ring_offsets.push(acc);
                }

                let coords = seq
                    .next_element_seed(ZigsSeed {
                        nr: total_rings,
                        ring_offsets: &ring_offsets,
                    })?
                    .ok_or_else(|| A::Error::custom("missing zigs"))?;

                Ok(TileMultiPolygon(MultiPolygon::from_raw_unchecked(
                    coords,
                    ring_offsets,
                    polygon_offsets,
                )))
            }
        }
        de.deserialize_struct("TileMultiPolygon", &["topo", "zigs"], V)
    }
}

// ---- Codec ----

#[derive(Copy, Clone)]
enum Codec {
    None,
    Lz4,
}

impl Codec {
    fn name(self) -> &'static str {
        match self {
            Codec::None => "none",
            Codec::Lz4 => "lz4",
        }
    }
    fn wrap(self, raw: Vec<u8>) -> Vec<u8> {
        match self {
            Codec::None => raw,
            Codec::Lz4 => lz4_flex::compress_prepend_size(&raw),
        }
    }
    fn unwrap<'a>(self, bytes: &'a [u8], scratch: &'a mut Vec<u8>) -> &'a [u8] {
        match self {
            Codec::None => bytes,
            Codec::Lz4 => {
                *scratch = lz4_flex::decompress_size_prepended(bytes).unwrap();
                scratch.as_slice()
            }
        }
    }
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> T {
    postcard::from_bytes(bytes).unwrap()
}

fn main() {
    let fixtures = ["fixtures/nl_plots.wkt", "fixtures/nl_zones.wkt"];
    for fixture in fixtures {
        run_fixture(fixture);
    }
}

fn run_fixture(fixture: &str) {
    let (wkt, flat, jagged) = load(fixture);
    let (flat_mp, jagged_mp) = match (flat, jagged) {
        (flatgeom::Geometry::MultiPolygon(f), geo_types::Geometry::MultiPolygon(j)) => (f, j),
        _ => {
            eprintln!("skip {fixture}: not a MultiPolygon");
            return;
        }
    };

    println!(
        "\n=== {} ({} polygons, {} byte WKT) ===",
        Path::new(fixture).file_name().unwrap().to_string_lossy(),
        flat_mp.len(),
        wkt.len()
    );
    println!(
        "{:<14} {:<6} {:<12} {:>12} {:>12} {:>12} {:>14}",
        "mode", "codec", "type", "size(B)", "encode(us)", "decode(us)", "enc+dec(us)"
    );
    println!("{}", "-".repeat(96));

    let (flat_i16, jagged_i16) = to_i16(&flat_mp, &jagged_mp);
    let jagged_polys_i: Vec<geo_types::Polygon<i16>> = jagged_i16.0.to_vec();
    let flat_polys_i: Vec<Polygon<[i16; 2]>> = flat_i16.iter().map(|v| v.to_owned()).collect();

    let tile_mp = TileMultiPolygon(flat_i16.clone());
    let tile_polys: Vec<TilePolygon> = flat_polys_i.iter().cloned().map(TilePolygon).collect();

    for &codec in &[Codec::None, Codec::Lz4] {
        let (size, enc, dec) = bench_bulk(&jagged_i16, codec);
        print_row("bulk", codec.name(), "jagged", size, enc, dec);
        let (size, enc, dec) = bench_bulk(&flat_i16, codec);
        print_row("bulk", codec.name(), "flat", size, enc, dec);
        let (size, enc, dec) = bench_bulk(&tile_mp, codec);
        print_row("bulk", codec.name(), "flat-tile", size, enc, dec);
    }
    println!();
    for &codec in &[Codec::None, Codec::Lz4] {
        let (size, enc, dec) = bench_per(&jagged_polys_i, codec);
        print_row("per-polygon", codec.name(), "jagged", size, enc, dec);
        let (size, enc, dec) = bench_per(&flat_polys_i, codec);
        print_row("per-polygon", codec.name(), "flat", size, enc, dec);
        let (size, enc, dec) = bench_per(&tile_polys, codec);
        print_row("per-polygon", codec.name(), "flat-tile", size, enc, dec);
    }
}

fn bench_bulk<T>(value: &T, codec: Codec) -> (usize, f64, f64)
where
    T: Serialize + DeserializeOwned,
{
    let bytes = codec.wrap(encode(value));
    let size = bytes.len();
    let enc = bench(|| {
        black_box(codec.wrap(encode(black_box(value))));
    });
    let dec = bench(|| {
        let mut scratch = Vec::new();
        let raw = codec.unwrap(&bytes, &mut scratch);
        black_box(decode::<T>(raw));
    });
    (size, micros(enc), micros(dec))
}

fn bench_per<I>(items: &[I], codec: Codec) -> (usize, f64, f64)
where
    I: Serialize + DeserializeOwned,
{
    let encoded: Vec<Vec<u8>> = items.iter().map(|i| codec.wrap(encode(i))).collect();
    let size: usize = encoded.iter().map(|b| b.len()).sum();
    let enc = bench(|| {
        for i in items {
            black_box(codec.wrap(encode(i)));
        }
    });
    let dec = bench(|| {
        let mut scratch = Vec::new();
        for b in &encoded {
            let raw = codec.unwrap(b, &mut scratch);
            black_box(decode::<I>(raw));
        }
    });
    (size, micros(enc), micros(dec))
}

fn bench<F: FnMut()>(mut f: F) -> Duration {
    let warm_end = Instant::now() + Duration::from_secs_f64(WARMUP_SECS);
    while Instant::now() < warm_end {
        f();
    }
    let target = Duration::from_secs_f64(BENCH_SECS);
    let start = Instant::now();
    let mut iters: u32 = 0;
    while start.elapsed() < target {
        f();
        iters += 1;
    }
    start.elapsed() / iters.max(1)
}

fn micros(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

fn to_i16(
    flat: &flatgeom::MultiPolygon<[f64; 2]>,
    jagged: &geo_types::MultiPolygon<f64>,
) -> (
    flatgeom::MultiPolygon<[i16; 2]>,
    geo_types::MultiPolygon<i16>,
) {
    let coords = flat.raw_coords();
    let mut x_min = f64::INFINITY;
    let mut y_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for c in coords {
        if c[0] < x_min {
            x_min = c[0];
        }
        if c[0] > x_max {
            x_max = c[0];
        }
        if c[1] < y_min {
            y_min = c[1];
        }
        if c[1] > y_max {
            y_max = c[1];
        }
    }
    let extent = (x_max - x_min).max(y_max - y_min);
    let scale = 4096.0 / extent;
    let q = move |x: f64, y: f64| -> (i16, i16) {
        (
            ((x - x_min) * scale).round() as i16,
            ((y - y_min) * scale).round() as i16,
        )
    };
    let flat_i16 = flat.transform(|c| {
        let (x, y) = q(c[0], c[1]);
        [x, y]
    });
    let jagged_i16 = map_jagged(jagged, |c| q(c.x, c.y));
    (flat_i16, jagged_i16)
}

fn map_jagged<T, U, F>(mp: &geo_types::MultiPolygon<T>, f: F) -> geo_types::MultiPolygon<U>
where
    T: geo_types::CoordNum,
    U: geo_types::CoordNum,
    F: Fn(&geo_types::Coord<T>) -> (U, U) + Copy,
{
    use geo_types::{Coord, LineString, MultiPolygon, Polygon};
    let q = |c: &Coord<T>| {
        let (x, y) = f(c);
        Coord { x, y }
    };
    let polys =
        mp.0.iter()
            .map(|p| {
                let exterior = LineString(p.exterior().0.iter().map(q).collect());
                let interiors = p
                    .interiors()
                    .iter()
                    .map(|ls| LineString(ls.0.iter().map(q).collect()))
                    .collect();
                Polygon::new(exterior, interiors)
            })
            .collect();
    MultiPolygon(polys)
}

fn load(
    fixture: &str,
) -> (
    String,
    flatgeom::Geometry<[f64; 2]>,
    geo_types::Geometry<f64>,
) {
    let wkt = fs::read_to_string(Path::new(fixture))
        .unwrap_or_else(|e| panic!("failed to read {fixture}: {e}"));
    let wkt_str = Wkt(wkt.as_str());
    let flat: flatgeom::Geometry<[f64; 2]> = wkt_str.to_flatgeom().expect("flatgeom parse");
    let jagged: geo_types::Geometry<f64> = wkt_str.to_geo().expect("geo_types parse");
    (wkt, flat, jagged)
}

fn print_row(mode: &str, codec: &str, kind: &str, size: usize, enc_us: f64, dec_us: f64) {
    println!(
        "{:<14} {:<6} {:<12} {:>12} {:>12.2} {:>12.2} {:>14.2}",
        mode,
        codec,
        kind,
        size,
        enc_us,
        dec_us,
        enc_us + dec_us
    );
}
