use crate::error::{FormatError, Result};
use crc::Crc;
use std::io::{Read, Write};

const CRC32C: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

// ---------- low-level helpers ----------

fn put_u8(b: &mut Vec<u8>, v: u8) { b.push(v); }
fn put_u16(b: &mut Vec<u8>, v: u16) { b.extend_from_slice(&v.to_le_bytes()); }
fn put_u32(b: &mut Vec<u8>, v: u32) { b.extend_from_slice(&v.to_le_bytes()); }
fn put_u64(b: &mut Vec<u8>, v: u64) { b.extend_from_slice(&v.to_le_bytes()); }
fn put_u128(b: &mut Vec<u8>, v: u128) { b.extend_from_slice(&v.to_le_bytes()); }
fn put_bytes(b: &mut Vec<u8>, v: &[u8]) { b.extend_from_slice(v); }

fn take_u8(cur: &mut &[u8]) -> Result<u8> {
    if cur.is_empty() { return Err(FormatError::Truncated); }
    let v = cur[0]; *cur = &cur[1..]; Ok(v)
}
fn take_u16(cur: &mut &[u8]) -> Result<u16> {
    if cur.len() < 2 { return Err(FormatError::Truncated); }
    let v = u16::from_le_bytes(cur[..2].try_into().unwrap()); *cur = &cur[2..]; Ok(v)
}
fn take_u32(cur: &mut &[u8]) -> Result<u32> {
    if cur.len() < 4 { return Err(FormatError::Truncated); }
    let v = u32::from_le_bytes(cur[..4].try_into().unwrap()); *cur = &cur[4..]; Ok(v)
}
fn take_u64(cur: &mut &[u8]) -> Result<u64> {
    if cur.len() < 8 { return Err(FormatError::Truncated); }
    let v = u64::from_le_bytes(cur[..8].try_into().unwrap()); *cur = &cur[8..]; Ok(v)
}
fn take_u128(cur: &mut &[u8]) -> Result<u128> {
    if cur.len() < 16 { return Err(FormatError::Truncated); }
    let v = u128::from_le_bytes(cur[..16].try_into().unwrap()); *cur = &cur[16..]; Ok(v)
}
fn take_fixed(cur: &mut &[u8], n: usize) -> Result<Vec<u8>> {
    if cur.len() < n { return Err(FormatError::Truncated); }
    let v = cur[..n].to_vec(); *cur = &cur[n..]; Ok(v)
}

// ---------- public types ----------

#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk_id: u128,
    pub modality: u32,
    pub fidelity: u8,
    pub sample_count: u32,
    pub raw_size: u64,
    pub codec_id: u32,
    pub transform_plan_index: u32,
    pub payload: Vec<u8>,
    pub content_hash: [u8; 32],
    pub dependency_id: u128,
    pub dependency_kind: u8,
}

#[derive(Debug, Clone)]
pub struct SampleField {
    pub field_id: u32,
    pub chunk_index: u32,
    pub byte_offset: u64,
    pub byte_length: u64,
}

#[derive(Debug, Clone)]
pub struct SampleEntry {
    pub sample_id: u64,
    pub primary_chunk_index: u32,
    pub fields: Vec<SampleField>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub dataset_name: String,
    pub created_at_unix_ms: u64,
    pub chunk_count: u64,
    pub sample_count: u64,
    pub raw_total_bytes: u64,
    pub encoded_total_bytes: u64,
    pub content_hash_dataset: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct Container {
    pub dataset_name: String,
    pub created_at_unix_ms: u64,
    pub chunks: Vec<Chunk>,
    pub samples: Vec<SampleEntry>,
    pub sample_count: u64,
}

impl Container {
    pub fn new(dataset_name: impl Into<String>) -> Self {
        Self { dataset_name: dataset_name.into(), created_at_unix_ms: 0, chunks: Vec::new(), samples: Vec::new(), sample_count: 0 }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub fn add_sample(&mut self, sample: SampleEntry) {
        self.samples.push(sample);
    }

    pub fn manifest(&self) -> Manifest {
        let raw_total: u64 = self.chunks.iter().map(|c| c.raw_size).sum();
        let enc_total: u64 = self.chunks.iter().map(|c| c.payload.len() as u64).sum();
        let sc: u64 = if self.samples.is_empty() {
            self.chunks.iter().map(|c| c.sample_count as u64).sum()
        } else {
            self.samples.len() as u64
        };
        Manifest {
            dataset_name: self.dataset_name.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            chunk_count: self.chunks.len() as u64,
            sample_count: sc,
            raw_total_bytes: raw_total,
            encoded_total_bytes: enc_total,
            content_hash_dataset: [0u8; 32],
        }
    }

    // ---------- serialization ----------

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let manifest = self.manifest();
        let manifest_bytes = serialize_manifest(&manifest);

        const DESC_SIZE: usize = 120;
        let n = self.chunks.len();
        let chunkdir_content_size = 8 + n * DESC_SIZE;

        // sample index bytes
        let sampleindex_bytes = serialize_sample_index(&self.samples);

        // compute offsets
        let header_size: usize = 128;
        let manifest_section_total = 24 + manifest_bytes.len();
        let manifest_offset = header_size;
        let chunkdir_offset = manifest_offset + manifest_section_total;
        let chunkdir_section_total = 24 + chunkdir_content_size;
        let sampleindex_offset = chunkdir_offset + chunkdir_section_total;
        let sampleindex_section_total = 24 + sampleindex_bytes.len();
        let payload_section_offset = sampleindex_offset + sampleindex_section_total;
        let payload_data_start = payload_section_offset + 24;
        let mut payload_offsets = Vec::with_capacity(n);
        let mut acc = 0usize;
        for c in &self.chunks {
            payload_offsets.push(payload_data_start + acc);
            acc += c.payload.len();
        }
        let payload_section_total = 24 + acc;
        let footer_offset = payload_section_offset + payload_section_total;

        // build header
        let mut header = vec![0u8; header_size];
        header[0..4].copy_from_slice(b"ODCF");
        header[4..6].copy_from_slice(&0u16.to_le_bytes());
        header[6..8].copy_from_slice(&1u16.to_le_bytes());
        header[8..12].copy_from_slice(&(header_size as u32).to_le_bytes());
        let mut feature_flags: u64 = 0;
        if !self.samples.is_empty() { feature_flags |= 1; }
        header[12..20].copy_from_slice(&feature_flags.to_le_bytes());
        header[36..44].copy_from_slice(&1u64.to_le_bytes());
        header[44..52].copy_from_slice(&(manifest_offset as u64).to_le_bytes());
        header[52..60].copy_from_slice(&(manifest_bytes.len() as u64).to_le_bytes());
        header[60..68].copy_from_slice(&(chunkdir_offset as u64).to_le_bytes());
        header[68..76].copy_from_slice(&(chunkdir_content_size as u64).to_le_bytes());
        header[76..84].copy_from_slice(&(sampleindex_offset as u64).to_le_bytes());
        header[84..92].copy_from_slice(&(sampleindex_bytes.len() as u64).to_le_bytes());
        header[92..100].copy_from_slice(&0u64.to_le_bytes()); // dict_dir_offset
        header[100..108].copy_from_slice(&0u64.to_le_bytes()); // dict_dir_size
        header[108..116].copy_from_slice(&(footer_offset as u64).to_le_bytes());
        let crc = CRC32C.checksum(&header[..116]);
        header[116..120].copy_from_slice(&crc.to_le_bytes());

        let mut out = Vec::with_capacity(footer_offset + 32);
        out.extend_from_slice(&header);

        // manifest section
        put_section_header(&mut out, 1, 1, manifest_bytes.len() as u64);
        out.extend_from_slice(&manifest_bytes);

        // chunk dir section
        put_section_header(&mut out, 2, 1, chunkdir_content_size as u64);
        put_u64(&mut out, n as u64);
        for (i, c) in self.chunks.iter().enumerate() {
            serialize_chunk_desc(&mut out, c, payload_offsets[i] as u64, c.payload.len() as u64);
        }

        // sample index section
        put_section_header(&mut out, 4, 1, sampleindex_bytes.len() as u64);
        out.extend_from_slice(&sampleindex_bytes);

        // payload section
        put_section_header(&mut out, 3, 1, acc as u64);
        for c in &self.chunks {
            out.extend_from_slice(&c.payload);
        }

        // footer (simple)
        let footer_start = out.len();
        put_u32(&mut out, 4u32); // section_count
        put_u32(&mut out, 1); put_u64(&mut out, manifest_offset as u64); put_u64(&mut out, manifest_bytes.len() as u64);
        put_u32(&mut out, 2); put_u64(&mut out, chunkdir_offset as u64); put_u64(&mut out, chunkdir_content_size as u64);
        put_u32(&mut out, 4); put_u64(&mut out, sampleindex_offset as u64); put_u64(&mut out, sampleindex_bytes.len() as u64);
        put_u32(&mut out, 3); put_u64(&mut out, payload_section_offset as u64); put_u64(&mut out, acc as u64);
        out.extend_from_slice(b"ODCF");
        put_u16(&mut out, 0);
        put_u16(&mut out, 1);
        let crc = CRC32C.checksum(&out[footer_start..]);
        put_u32(&mut out, crc);

        Ok(out)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 128 { return Err(FormatError::Truncated); }
        if &data[0..4] != b"ODCF" { return Err(FormatError::BadMagic); }
        let major = u16::from_le_bytes(data[4..6].try_into().unwrap());
        let minor = u16::from_le_bytes(data[6..8].try_into().unwrap());
        if major != 0 { return Err(FormatError::BadVersion(major, minor)); }
        let header_crc = u32::from_le_bytes(data[116..120].try_into().unwrap());
        if CRC32C.checksum(&data[..116]) != header_crc {
            return Err(FormatError::CrcMismatch);
        }
        let manifest_offset = u64::from_le_bytes(data[44..52].try_into().unwrap()) as usize;
        let manifest_size = u64::from_le_bytes(data[52..60].try_into().unwrap()) as usize;
        let chunkdir_offset = u64::from_le_bytes(data[60..68].try_into().unwrap()) as usize;
        let chunkdir_size = u64::from_le_bytes(data[68..76].try_into().unwrap()) as usize;
        let sampleindex_offset = u64::from_le_bytes(data[76..84].try_into().unwrap()) as usize;
        let sampleindex_size = u64::from_le_bytes(data[84..92].try_into().unwrap()) as usize;
        let _dict_offset = u64::from_le_bytes(data[92..100].try_into().unwrap());
        let footer_offset = u64::from_le_bytes(data[108..116].try_into().unwrap()) as usize;

        // manifest
        let mut cur = &data[manifest_offset..];
        let _stype = take_u32(&mut cur)?;
        let _sver = take_u16(&mut cur)?;
        let _sflags = take_u16(&mut cur)?;
        let ssize = take_u64(&mut cur)? as usize;
        let _scrc = take_u32(&mut cur)?;
        let _sres = take_u32(&mut cur)?;
        let manifest_bytes = &data[manifest_offset + 24 .. manifest_offset + 24 + manifest_size];
        let manifest = parse_manifest(manifest_bytes)?;

        // chunk dir
        let mut cur = &data[chunkdir_offset..];
        let _stype = take_u32(&mut cur)?;
        let _sver = take_u16(&mut cur)?;
        let _sflags = take_u16(&mut cur)?;
        let _ssize = take_u64(&mut cur)?;
        let _scrc = take_u32(&mut cur)?;
        let _sres = take_u32(&mut cur)?;
        let mut cur2 = &data[chunkdir_offset + 24 ..];
        let count = take_u64(&mut cur2)? as usize;
        let mut chunks = Vec::with_capacity(count);
        for _ in 0..count {
            let desc_bytes = take_fixed(&mut cur2, 120)?;
            let mut dc = &desc_bytes[..];
            let chunk_id = take_u128(&mut dc)?;
            let modality = take_u32(&mut dc)?;
            let fidelity = take_u8(&mut dc)?;
            let _flags = take_u8(&mut dc)?;
            let _reserved = take_u16(&mut dc)?;
            let sample_count = take_u32(&mut dc)?;
            let raw_size = take_u64(&mut dc)?;
            let _encoded_size = take_u64(&mut dc)?;
            let codec_id = take_u32(&mut dc)?;
            let transform_plan_index = take_u32(&mut dc)?;
            let payload_offset = take_u64(&mut dc)? as usize;
            let payload_size = take_u64(&mut dc)? as usize;
            let mut hash = [0u8; 32];
            let hb = take_fixed(&mut dc, 32)?;
            hash.copy_from_slice(&hb);
            let dependency_id = take_u128(&mut dc)?;
            let dependency_kind = take_u8(&mut dc)?;
            let _r1 = take_u8(&mut dc)?;
            let _r2 = take_u16(&mut dc)?;
            if payload_offset + payload_size > data.len() {
                return Err(FormatError::Truncated);
            }
            let payload = data[payload_offset..payload_offset+payload_size].to_vec();
            chunks.push(Chunk {
                chunk_id, modality, fidelity, sample_count, raw_size,
                codec_id, transform_plan_index, payload, content_hash: hash,
                dependency_id, dependency_kind,
            });
        }

        // sample index
        let mut samples = Vec::new();
        if sampleindex_size > 0 {
            let mut cur = &data[sampleindex_offset..];
            let _stype = take_u32(&mut cur)?;
            let _sver = take_u16(&mut cur)?;
            let _sflags = take_u16(&mut cur)?;
            let _ssize = take_u64(&mut cur)?;
            let _scrc = take_u32(&mut cur)?;
            let _sres = take_u32(&mut cur)?;
            let mut cur2 = &data[sampleindex_offset + 24 ..];
            let scount = take_u64(&mut cur2)? as usize;
            for _ in 0..scount {
                let sid = take_u64(&mut cur2)?;
                let pc = take_u32(&mut cur2)?;
                let fc = take_u16(&mut cur2)? as usize;
                let mut fields = Vec::with_capacity(fc);
                for _ in 0..fc {
                    let fid = take_u32(&mut cur2)?;
                    let ci = take_u32(&mut cur2)?;
                    let off = take_u64(&mut cur2)?;
                    let ln = take_u64(&mut cur2)?;
                    fields.push(SampleField { field_id: fid, chunk_index: ci, byte_offset: off, byte_length: ln });
                }
                samples.push(SampleEntry { sample_id: sid, primary_chunk_index: pc, fields });
            }
        }

        Ok(Container {
            dataset_name: manifest.dataset_name,
            created_at_unix_ms: manifest.created_at_unix_ms,
            chunks,
            samples,
            sample_count: manifest.sample_count,
        })
    }

    pub fn write_file(&self, path: &std::path::Path) -> Result<()> {
        let bytes = self.to_bytes()?;
        let mut f = std::fs::File::create(path)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        Ok(())
    }

    pub fn read_file(path: &std::path::Path) -> Result<Self> {
        let mut f = std::fs::File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Self::from_bytes(&buf)
    }

    pub fn verify_basic(&self) -> Result<()> {
        let m = self.manifest();
        if m.chunk_count != self.chunks.len() as u64 {
            return Err(FormatError::InvalidField("chunk count mismatch".into()));
        }
        Ok(())
    }

    /// Resolve a sample by id.
    pub fn find_sample(&self, sample_id: u64) -> Option<&SampleEntry> {
        self.samples.iter().find(|s| s.sample_id == sample_id)
    }
}

fn put_section_header(b: &mut Vec<u8>, stype: u32, version: u16, size: u64) {
    put_u32(b, stype);
    put_u16(b, version);
    put_u16(b, 0);
    put_u64(b, size);
    put_u32(b, 0);
    put_u32(b, 0);
}

fn serialize_manifest(m: &Manifest) -> Vec<u8> {
    let mut b = Vec::new();
    let nb = m.dataset_name.as_bytes();
    put_u32(&mut b, nb.len() as u32);
    put_bytes(&mut b, nb);
    put_u64(&mut b, m.created_at_unix_ms);
    put_u64(&mut b, m.chunk_count);
    put_u64(&mut b, m.sample_count);
    put_u64(&mut b, m.raw_total_bytes);
    put_u64(&mut b, m.encoded_total_bytes);
    put_bytes(&mut b, &m.content_hash_dataset);
    b
}

fn parse_manifest(mut data: &[u8]) -> Result<Manifest> {
    let nlen = take_u32(&mut data)? as usize;
    let nb = take_fixed(&mut data, nlen)?;
    let dataset_name = String::from_utf8(nb).map_err(|_| FormatError::InvalidField("name".into()))?;
    let created_at_unix_ms = take_u64(&mut data)?;
    let chunk_count = take_u64(&mut data)?;
    let sample_count = take_u64(&mut data)?;
    let raw_total_bytes = take_u64(&mut data)?;
    let encoded_total_bytes = take_u64(&mut data)?;
    let hb = take_fixed(&mut data, 32)?;
    let mut content_hash_dataset = [0u8; 32];
    content_hash_dataset.copy_from_slice(&hb);
    Ok(Manifest {
        dataset_name, created_at_unix_ms, chunk_count, sample_count,
        raw_total_bytes, encoded_total_bytes, content_hash_dataset,
    })
}

fn serialize_sample_index(samples: &[SampleEntry]) -> Vec<u8> {
    let mut b = Vec::new();
    put_u64(&mut b, samples.len() as u64);
    for s in samples {
        put_u64(&mut b, s.sample_id);
        put_u32(&mut b, s.primary_chunk_index);
        put_u16(&mut b, s.fields.len() as u16);
        for f in &s.fields {
            put_u32(&mut b, f.field_id);
            put_u32(&mut b, f.chunk_index);
            put_u64(&mut b, f.byte_offset);
            put_u64(&mut b, f.byte_length);
        }
    }
    b
}

fn serialize_chunk_desc(b: &mut Vec<u8>, c: &Chunk, payload_offset: u64, payload_size: u64) {
    put_u128(b, c.chunk_id);
    put_u32(b, c.modality);
    put_u8(b, c.fidelity);
    put_u8(b, 0); // flags
    put_u16(b, 0); // reserved
    put_u32(b, c.sample_count);
    put_u64(b, c.raw_size);
    put_u64(b, c.payload.len() as u64);
    put_u32(b, c.codec_id);
    put_u32(b, c.transform_plan_index);
    put_u64(b, payload_offset);
    put_u64(b, payload_size);
    put_bytes(b, &c.content_hash);
    put_u128(b, c.dependency_id);
    put_u8(b, c.dependency_kind);
    put_u8(b, 0);
    put_u16(b, 0);
}
