use halo2_proofs::halo2curves::bn256::Fr;
use snark_verifier_sdk::CircuitExt;

use super::MockChunkCircuit;

impl CircuitExt<Fr> for MockChunkCircuit {
    /// 64 elements from digest
    fn num_instance(&self) -> Vec<usize> {
        vec![64]
    }

    /// return vec![data hash | public input hash]
    fn instances(&self) -> Vec<Vec<Fr>> {
        vec![self
            .chunk
            .data_hash
            .as_bytes()
            .iter()
            .chain(self.chunk.public_input_hash().as_bytes().iter())
            .map(|&x| Fr::from(x as u64))
            .collect()]
    }
}