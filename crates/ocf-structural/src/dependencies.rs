use crate::content_address::id_from_hash;
use ocf_core::OcfError;
use ocf_format::Chunk;

pub const MAX_DEPTH: u8 = 8;

/// Validate dependency graph: existence, max depth, no cycles.
pub fn validate_dependencies(chunks: &[Chunk]) -> Result<(), OcfError> {
    // map id -> index
    let mut by_id: std::collections::HashMap<u128, usize> = std::collections::HashMap::new();
    for (i, c) in chunks.iter().enumerate() {
        by_id.insert(id_from_hash(&c.content_hash), i);
    }

    // adjacency: chunk index -> dependency chunk index (if any)
    let mut dep: Vec<Option<usize>> = vec![None; chunks.len()];
    for (i, c) in chunks.iter().enumerate() {
        if c.dependency_kind != 0 {
            match by_id.get(&c.dependency_id) {
                Some(&j) => dep[i] = Some(j),
                None => return Err(OcfError::DependencyMissing(format!("chunk {} dep {}", i, c.dependency_id))),
            }
        }
    }

    let mut state = vec![0u8; chunks.len()];
    fn dfs(node: usize, depth: u8, dep: &[Option<usize>], state: &mut [u8]) -> Result<(), OcfError> {
        if depth > MAX_DEPTH {
            return Err(OcfError::DepthExceeded(depth));
        }
        state[node] = 1;
        if let Some(d) = dep[node] {
            if state[d] == 1 {
                return Err(OcfError::DependencyCycle);
            }
            if state[d] == 0 {
                dfs(d, depth + 1, dep, state)?;
            }
        }
        state[node] = 2;
        Ok(())
    }
    for i in 0..chunks.len() {
        if state[i] == 0 {
            dfs(i, 0, &dep, &mut state)?;
        }
    }
    Ok(())
}
