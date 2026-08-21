//! Stable content fingerprints used where an audit record names a complete derived artifact.
//!
//! D17 keeps the core dependency-free. Keeping SHA-256 here avoids delegating the archive record to
//! a platform command whose availability and output format would become part of the contract.

pub fn sha256(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, bytes) in chunk.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let first = h
                .wrapping_add(sum1)
                .wrapping_add(choose)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let second = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(first);
            d = c;
            c = b;
            b = a;
            a = first.wrapping_add(second);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    state.iter().map(|word| format!("{word:08x}")).collect()
}

/// SHA-256 over canonical JSON. Object keys are sorted recursively; callers sort collections whose
/// semantics are set-like before constructing the value.
pub fn canonical_json(value: &crate::json::Json) -> String {
    fn canonical(value: &crate::json::Json) -> crate::json::Json {
        use crate::json::Json;
        match value {
            Json::Arr(items) => Json::Arr(items.iter().map(canonical).collect()),
            Json::Obj(fields) => {
                let mut fields = fields
                    .iter()
                    .map(|(key, value)| (key.clone(), canonical(value)))
                    .collect::<Vec<_>>();
                fields.sort_by(|left, right| left.0.cmp(&right.0));
                Json::Obj(fields)
            }
            value => value.clone(),
        }
    }

    canonical(value).to_string_pretty()
}

pub fn canonical_sha256(value: &crate::json::Json) -> String {
    format!("sha256:{}", sha256(canonical_json(value).as_bytes()))
}

pub fn check_fingerprint(
    check: &crate::verification::Check,
    implementations: &[crate::model::CheckImplementation],
) -> String {
    use crate::json::Json;
    let mut implementations = implementations
        .iter()
        .filter(|implementation| implementation.check == check.id)
        .map(|implementation| {
            Json::obj(vec![
                ("identity", Json::str(implementation.semantic_identity())),
                (
                    "source_fingerprint",
                    Json::str(&implementation.source_fingerprint),
                ),
            ])
        })
        .collect::<Vec<_>>();
    implementations.sort_by_key(canonical_json);
    implementations.dedup();
    canonical_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-check-fingerprint")),
        ("version", Json::Num(1.0)),
        ("id", Json::str(&check.id)),
        (
            "methods",
            Json::Arr(check.methods.iter().map(Json::str).collect()),
        ),
        ("terminal", Json::str(&check.terminal)),
        ("implementations", Json::Arr(implementations)),
    ]))
}

pub fn policy_fingerprint(policy: &crate::verification::QualificationPolicy) -> String {
    canonical_sha256(&crate::verification::policy_json(policy))
}

pub fn binding_fingerprint(
    binding: &crate::verification::EvidenceBinding,
    claim_digest: &str,
    policy_digest: &str,
) -> String {
    use crate::json::Json;
    let mut challenge_domain = binding
        .challenge_domain
        .iter()
        .map(|domain| domain.name())
        .collect::<Vec<_>>();
    challenge_domain.sort();
    challenge_domain.dedup();
    canonical_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-evidence-binding-fingerprint")),
        ("version", Json::Num(1.0)),
        ("id", Json::str(&binding.id)),
        ("check", Json::str(&binding.check)),
        ("claim_digest", Json::str(claim_digest)),
        ("proposition", Json::str(&binding.proposition)),
        (
            "form",
            Json::obj(vec![
                ("scope", Json::str(binding.scope.name())),
                ("quantification", Json::str(binding.quantification.name())),
                ("oracle", Json::str(binding.oracle.name())),
            ]),
        ),
        (
            "challenge_domain",
            Json::Arr(challenge_domain.into_iter().map(Json::str).collect()),
        ),
        ("qualification_policy_digest", Json::str(policy_digest)),
    ]))
}

pub fn context_fingerprint(binding: &crate::verification::EvidenceBinding) -> String {
    canonical_sha256(&crate::verification::context_json(&binding.context))
}

pub fn qualification_fingerprint(
    check_fingerprint: &str,
    binding_fingerprint: &str,
    context_fingerprint: &str,
) -> String {
    use crate::json::Json;
    canonical_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-qualification-fingerprint")),
        ("version", Json::Num(1.0)),
        ("check_fingerprint", Json::str(check_fingerprint)),
        ("binding_fingerprint", Json::str(binding_fingerprint)),
        ("context_fingerprint", Json::str(context_fingerprint)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::sha256;

    #[test]
    fn matches_the_standard_empty_input_vector() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
