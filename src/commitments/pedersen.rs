//! Pedersen commitment over elliptic curves.

use crate::commitments::{Commitment, CommitmentError};
use crate::utils::{curve::CurvePointProjective, integer_to_bigint};
use rand::{CryptoRng, RngCore};
use rug::Integer;

use crate::utils::SerdeAs;
use serde::{Serialize, Deserialize};
use serde_with::serde_as;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

#[serde_as]
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableCurvePointProjective<P: CurvePointProjective>(
    #[serde_as(as = "SerdeAs")] pub P
);
#[derive(Clone)]
pub struct PedersenCommitment<P: CurvePointProjective> {
    pub g: P,
    pub h: P,
}

impl<P: CurvePointProjective> PedersenCommitment<P> {
    pub fn setup<R: RngCore + CryptoRng>(rng: &mut R) -> PedersenCommitment<P> {
        PedersenCommitment {
            g: P::rand(rng),
            h: P::rand(rng),
        }
    }

    pub fn new(g: &P, h: &P) -> PedersenCommitment<P> {
        PedersenCommitment {
            g: g.clone(),
            h: h.clone(),
        }
    }
}
impl<P: CurvePointProjective> Commitment for PedersenCommitment<P> {
    type Instance = SerializableCurvePointProjective<P>;

    fn commit(
        &self,
        value: &Integer,
        randomness: &Integer,
    ) -> Result<Self::Instance, CommitmentError> {
        let v = integer_to_bigint::<P>(value);
        let r = integer_to_bigint::<P>(randomness);
        Ok(SerializableCurvePointProjective(self.g.mul(&v).add(&self.h.mul(&r))))
    }

    fn open(
        &self,
        commitment: &Self::Instance,
        value: &Integer,
        randomness: &Integer,
    ) -> Result<(), CommitmentError> {
        let expected = self
            .g
            .mul(&integer_to_bigint::<P>(value))
            .add(&self.h.mul(&integer_to_bigint::<P>(randomness)));
        if expected == commitment.0 {
            Ok(())
        } else {
            Err(CommitmentError::WrongOpening)
        }
    }
}

#[cfg(all(test, feature = "arkworks"))]
mod test {
    use super::PedersenCommitment;
    use crate::commitments::Commitment;
    use ark_bls12_381::G1Projective;
    use rand::thread_rng;
    use rug::Integer;

    #[test]
    fn test_simple_commitment() {
        let mut rng = thread_rng();

        let value = Integer::from(2);
        let randomness = Integer::from(5);
        let pedersen = PedersenCommitment::<G1Projective>::setup(&mut rng);
        let commitment = pedersen.commit(&value, &randomness).unwrap();
        pedersen.open(&commitment, &value, &randomness).unwrap();
        let wrong_value = Integer::from(5);
        pedersen
            .open(&commitment, &wrong_value, &randomness)
            .unwrap_err();
        let wrong_randomness = Integer::from(7);
        pedersen
            .open(&commitment, &value, &wrong_randomness)
            .unwrap_err();
        pedersen
            .open(&commitment, &wrong_value, &wrong_randomness)
            .unwrap_err();
    }
}
