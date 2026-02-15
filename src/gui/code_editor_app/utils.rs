pub(super) fn build_gutter_text(line_count: usize) -> String {
    (1..=line_count)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn copy_trimmed_nonzero_slice(src: &[u8], dest: &mut [u8]) -> Result<(), String> {
    if src.len() > dest.len() {
        return Err(format!(
            "Źródło ({}) jest większe niż pamięć ({})",
            src.len(),
            dest.len()
        ));
    }

    let start = src.iter().position(|&b| b != 0);
    let end = src.iter().rposition(|&b| b != 0);
    let (Some(start), Some(end)) = (start, end) else {
        return Ok(());
    };

    dest[start..=end].copy_from_slice(&src[start..=end]);
    Ok(())
}

pub(super) fn normalize_output_chunk(chunk: &str) -> String {
    chunk.replace('\t', "    ")
}
