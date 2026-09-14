//! Comparación de versiones `x.y.z` (se ignora un prefijo `v` y lo que vaya tras `-`).

fn partes(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .split('-')
        .next()
        .unwrap_or_default()
        .split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect()
}

/// `a > b`
pub fn es_mayor(a: &str, b: &str) -> bool {
    let (mut x, mut y) = (partes(a), partes(b));
    let n = x.len().max(y.len());
    x.resize(n, 0);
    y.resize(n, 0);
    x > y
}

#[cfg(test)]
mod tests {
    use super::es_mayor;

    #[test]
    fn compara() {
        assert!(es_mayor("1.1.0", "1.0.9"));
        assert!(es_mayor("v1.10", "1.9.9"));
        assert!(!es_mayor("1.0", "1.0.0"));
        assert!(!es_mayor("1.0.0", "1.0.1"));
    }
}
