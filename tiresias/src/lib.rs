// Author: dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear

pub use ::group::ComputationalSecuritySizedNumber;
#[cfg(feature = "benchmarking")]
use criterion::criterion_group;
use crypto_bigint::{
    modular::runtime_mod::{DynResidue, DynResidueParams},
    Concat, Limb, Uint, U1024,
};
pub use decryption_key::DecryptionKey;
pub use decryption_key_share::DecryptionKeyShare;
pub use encryption_key::EncryptionKey;
pub use error::{Error, ProtocolError, Result, SanityCheckError};
pub use group::{
    CiphertextSpaceGroupElement, CiphertextSpacePublicParameters, CiphertextSpaceValue,
    PlaintextSpaceGroupElement, PlaintextSpacePublicParameters, PlaintextSpaceValue,
    RandomnessSpaceGroupElement, RandomnessSpacePublicParameters, RandomnessSpaceValue,
    CIPHERTEXT_SPACE_SCALAR_LIMBS, PLAINTEXT_SPACE_SCALAR_LIMBS, RANDOMNESS_SPACE_SCALAR_LIMBS,
};

mod batch_verification;
mod decryption_key;
pub mod decryption_key_share;
pub mod encryption_key;
mod error;
mod group;
pub mod proofs;
pub mod secret_sharing;

// Being overly-conservative here
pub type StatisticalSecuritySizedNumber = ComputationalSecuritySizedNumber;

/// A type alias for an unsigned integer of the size of the Paillier large prime factors.
/// Set to a U1024 for 112-bit security.
pub type LargePrimeSizedNumber = U1024;

/// A type alias for an unsigned integer of the size of the Paillier associated bi-prime `n` ($N$)
/// (double the size of the Paillier large prime factors). Set to a U2048 for 112-bit security.
pub type LargeBiPrimeSizedNumber = <LargePrimeSizedNumber as Concat>::Output;

/// A type alias for an unsigned integer of the size of the Paillier modulus ($N^2$) (double the
/// size of the Paillier associated bi-prime `n` ($N$)). Set to a U4096 for 112-bit security.
pub type PaillierModulusSizedNumber = <LargeBiPrimeSizedNumber as Concat>::Output;

pub(crate) type PaillierRingElement = DynResidue<{ PaillierModulusSizedNumber::LIMBS }>;
pub(crate) type PaillierPlaintextRingElement = DynResidue<{ LargeBiPrimeSizedNumber::LIMBS }>;

const fn secret_sharing_polynomial_coefficient_size_upper_bound(
    number_of_parties: usize,
    threshold: usize,
) -> usize {
    // Account for summing up `number_of_parties` shamir shares (one from each party)
    factorial_upper_bound(number_of_parties)
        + 2 * const_log(threshold)
        + 2
        + PaillierModulusSizedNumber::BITS
        + StatisticalSecuritySizedNumber::BITS
        + const_log(number_of_parties)
}

const fn secret_key_share_size_upper_bound(number_of_parties: usize, threshold: usize) -> usize {
    secret_sharing_polynomial_coefficient_size_upper_bound(number_of_parties, threshold)
        + threshold * const_log(number_of_parties)
        + 1
}

// Must use `const` functions for macros, unfortunately `ilog2` returns `u32` and we don't have a
// `const` transition to `usize`
const fn const_log(n: usize) -> usize {
    let mut power = 1;
    let mut counter = 0;

    while power < n {
        power *= 2;
        counter += 1;
    }

    counter
}

const fn factorial_upper_bound(number_of_parties: usize) -> usize {
    // See https://math.stackexchange.com/questions/55709/how-to-prove-this-approximation-of-logarithm-of-factorial
    // This expands to $(n+1)log(n+1) - n$ when further bounding $e$ to its floor $2$.
    (number_of_parties + 1) * const_log(number_of_parties + 1) - number_of_parties
}

const fn adjusted_lagrange_coefficient_sized_number(
    number_of_parties: usize,
    threshold: usize,
) -> usize {
    // An upper bound for:
    //  $ 2{n\choose j}\Pi_{j'\in [n] \setminus S} |j'-j| $
    (number_of_parties - threshold) * const_log(number_of_parties)
        + 4 * number_of_parties
        + 2 * threshold
}

pub const MAX_PLAYERS: usize = 1024;
pub const SECRET_SHARING_POLYNOMIAL_COEFFICIENT_SIZE_UPPER_BOUND: usize =
    secret_sharing_polynomial_coefficient_size_upper_bound(MAX_PLAYERS, MAX_PLAYERS);
pub const SECRET_KEY_SHARE_SIZE_UPPER_BOUND: usize =
    secret_key_share_size_upper_bound(MAX_PLAYERS, MAX_PLAYERS);
pub const ADJUSTED_LAGRANGE_COEFFICIENT_SIZE_UPPER_BOUND: usize =
    adjusted_lagrange_coefficient_sized_number(MAX_PLAYERS, MAX_PLAYERS);

pub type SecretKeyShareSizedNumber =
    Uint<{ SECRET_KEY_SHARE_SIZE_UPPER_BOUND.next_power_of_two() / Limb::BITS }>;

pub(crate) type ProofOfEqualityOfDiscreteLogsRandomnessSizedNumber = Uint<
    {
        (SECRET_KEY_SHARE_SIZE_UPPER_BOUND + 2 * ComputationalSecuritySizedNumber::BITS)
            .next_power_of_two()
            / Limb::BITS
    },
>;

pub type AdjustedLagrangeCoefficientSizedNumber =
    Uint<{ ADJUSTED_LAGRANGE_COEFFICIENT_SIZE_UPPER_BOUND.next_power_of_two() / Limb::BITS }>;

/// Retrieve the minimal natural number in the congruence class.
pub(crate) trait AsNaturalNumber<T> {
    fn as_natural_number(&self) -> T;
}

/// Represent this natural number as the minimal member of the congruence class.
/// I.e., as a member of the ring $\mathbb{Z}_{n}$
pub(crate) trait AsRingElement<T> {
    fn as_ring_element(&self, n: &Self) -> T;
}

impl AsNaturalNumber<PaillierModulusSizedNumber> for PaillierRingElement {
    fn as_natural_number(&self) -> PaillierModulusSizedNumber {
        self.retrieve()
    }
}

impl AsRingElement<PaillierRingElement> for PaillierModulusSizedNumber {
    fn as_ring_element(&self, n: &Self) -> PaillierRingElement {
        let ring_params = DynResidueParams::new(n);
        DynResidue::new(self, ring_params)
    }
}

impl AsNaturalNumber<LargeBiPrimeSizedNumber> for PaillierPlaintextRingElement {
    fn as_natural_number(&self) -> LargeBiPrimeSizedNumber {
        self.retrieve()
    }
}

impl AsRingElement<PaillierPlaintextRingElement> for LargeBiPrimeSizedNumber {
    fn as_ring_element(&self, n: &Self) -> PaillierPlaintextRingElement {
        let ring_params = DynResidueParams::new(n);
        DynResidue::new(self, ring_params)
    }
}

#[cfg(any(test, feature = "test_exports"))]
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod test_exports {
    use crypto_bigint::NonZero;
    pub use decryption_key_share::test_exports::*;
    use rstest::rstest;

    use super::*;

    pub(crate) const N2: PaillierModulusSizedNumber = PaillierModulusSizedNumber::from_be_hex("5960383b5378ad0607f0f270ce7fb6dcaba6506f9fc56deeffaf605c9128db8ccf063e2e8221a8bdf82c027741a0303b08eb71fa6225a03df18f24c473dc6d4d3d30eb9c52a233bbfe967d04011b95e8de5bc482c3c217bcfdeb4df6f57af6ba9c6d66c69fb03a70a41fe1e87975c85343ef7d572ca06a0139706b23ed2b73ad72cb1b7e2e41840115651897c8757b3da9af3a60eebb6396ffd193738b4f04aa6ece638cef1bf4e9c45cf57f8debeda8598cbef732484752f5380737ba75ee00bf1b146817b9ab336d0ce5540395377347c653d1c9d272127ff12b9a0721b8ef13ecd8a8379f1b9a358de2af2c4cd97564dbd5328c2fc13d56ee30c8a101d333f5406afb1f4417b49d7a629d5076726877df11f05c998ae365e374a0141f0b99802214532c97c1ebf9faf6e277a8f29dbd8f3eab72266e60a77784249694819e42877a5e826745c97f84a5f37002b74d83fc064cf094be0e706a6710d47d253c4532e6aa4a679a75fa1d860b39085dab03186c67248e6c92223682f58bd41b67143e299329ce3a8045f3a0124c3d0ef9f0f49374d89b37d9c3321feb2ab4117df4f68246724ce41cd765326457968d848afcc0735531e5de7fea88cf2eb35ac68710c6e79d5ad25df6c0393c0267f56e8eac90a52637abe3e606769e70b20560eaf70e0d531b11dca299104fa933f887d85fb5f72386c196e40f559baee356b9");
    pub(crate) const PLAINTEXT: LargeBiPrimeSizedNumber = LargeBiPrimeSizedNumber::from_be_hex("23f6379f4b0435dd50c0eb12454495c99db09aed97fe498c0dba7c51f6c52ab7b8d8ba47896ee0c43d567a1b3611cb2d53ee74574acc9c4520106c0f6e5d0376817febb477bb729405387b6ae6e213b3b34c0eb0cbe5dff49452979ab7f0b514560b5c9b659732efd0d67a3d7b7512a5d97f1bde1c2263f741838a7c62d78133396715c9568c0524e20a3147cda4510ef2f32cefa6fb92caf3a26da63aba3693efce706303fe399b6c86664b1ccaa9fe6e1505d82c4dd9b0a60ea29ec88a91bf2656a3927ad39d561bfe4009f94398a9a7782383f063adeb922275efd950ef3739dee7854bbf93f939a947e3aec7344135e6b0623aff35e802311c10ede8b0d4");
    pub(crate) const RANDOMNESS: LargeBiPrimeSizedNumber = LargeBiPrimeSizedNumber::from_be_hex("4aba7692cfc2e1a30d46dc393c4d406837df82896da97268b377b8455ce9364d93ff7d0c051eed84f2335eeae95eaf5182055a9738f62d37d06cf4b24c663006513c823418d63db307a96a1ec6c4089df23a7cc69c4c64f914420955a3468d93087feedea153e05d94d184e823796dd326f8f6444405665b9a6af3a5fedf4d0e787792667e6e73e4631ea2cbcf7baa58fff7eb25eb739c31fadac1cd066d97bcd822af06a1e4df4a2ab76d252ddb960bbdc333fd38c912d27fa775e598d856a87ce770b1379dde2fbfce8d82f8692e7e1b33130d556c97b690d0b5f7a2f8652b79a8f07a35d3c4b9074be68daa04f13e7c54124d9dd4fe794a49375131d9c0b1");
    pub(crate) const CIPHERTEXT: PaillierModulusSizedNumber = PaillierModulusSizedNumber::from_be_hex("0d1a2a781bf90133552b120beb2745bbe02b47cc4e5cc65b6eb5294770bd44b52ce581c4aec199687283360ab0c46bb3f0bb33733dbbf2d7e95a7c600ed20e990e8c3133f7ec238c0b47882363df7748757717443a3d1f9e85f0fb27e665844f591a0f922f42436688a72a71bdf7e93c764a84aff5b813c034787f5cf35a7102fe3be8c670ac26b83b08dabca47d9156ce09d7349ac73d269b7355d5266720654b83b09857add1a6c0be4677115f461ea15907e1472d3d7dcde351f9eff7e43968ae7012a67eeca940c25d3dd5694c5bbf1ed702bfd2094e424bb17bbf00270ded29320cd2e50af2283121ecf5f8593de49b18e465f3b1e1a39daca4d7382e4a610bdbd21dfd343108085b6e2c743f295df3785d3766b56c36efc0ea10ba3de8c16c43fcc051e7c27d835a481c0fdd48819ca9398043689027b00b275ca048018788a5133b280981afb0d6da7e64f3cf5f9e39e501fe7b80807b872ece22f6e4b6b0d8279656ceef614c87ce7ee314a339ef44c3adc4f5e5451b2649c215a358c0682095e19d52ed454d5f4e364397928996823cb02c61f8304561cb21e3bd0f4399f283b0b1ded686ace5dc653b240760c6437323fab45418b904d2eef8ab0639b4cba7cccee58f471413505ca0f8bb5a859769ad9465ddac949d22114cacaeadb72962816c49f50adc6338da7a54bdda29f8e6e667d832bd9c9f9841be8b18");
    pub(crate) const WITNESS: SecretKeyShareSizedNumber = SecretKeyShareSizedNumber::from_be_hex("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002F8B8B2E3021C0797DA11EC2328DBD27AC756B7244083977EDF04B4F3E5A961173B9E0A61C88A5B5F28A9CCC09662C7465F09E500C7EFD5F901CA266D4FE000371D1C155D5A6A3E8B6CF7285D8FC9DFC28B5A514E61BE42C72387A90071E0DE61EE2EAD808978F02AED94AE02F51192C7A074A726EB19066FC76162054BD1D2BAEAB9C8B0914E8BD4DDB11E62F23634E3D2A33C3DFDD846C68A0A583F0BA1C72AAF090D7E8F04BB2F4E4B71B95339D3B7A936E170CF87BB0CE09C01852CA4438F2903A6D321768839CBAE020AC8112575C7B5C0927F14F717827FC83D030725CEE33E93799D407F8C946310311D3AD00276C9518BBA49C49A13B126496EEA3F50F3979681849DB27767499B99CFAD93CBA5EDCFFED2C4517E659E3812FF07239F403D64E86A99FFCF0DE3D807FE3122A6186B39FFC072D48036626F6ED57D0DE55A267D88AE33B2202F770EB212ED95A6014C2D649182AB48B8A9377192556A2B428D0F3E2300462BFE77D4A8617540665ECB8C25599D5151470DE53ECD4F841FCB7CEF6A5AA8258A278A075245A6AC6B68D319FF97D16BDF10055015E2E4A578559CE7E69ECDCB640D9601D1E0E222F5FC8BD598F1238D1D37EF6B954F151F54168F233F3F996AD630FEAD0B3B4FE33A1A9C82D304F2B47DA3E543586FD3678ACFD731C23622730F1D201312C6F2F64A4523DC73A6EBD48E031D8D5C844A5B7B916817BFF784562591F6F74511DBFFAF8FB");

    fn factorial(num: u16) -> u64 {
        (1u64..=u64::from(num)).product()
    }

    #[test]
    fn as_natural_number_and_as_natural_number_circles_correctly() {
        let x = PaillierModulusSizedNumber::from_be_hex("19BB1B2E0015AA04BEE4F8321819448A2C809DF799C6627668DAA936E3A367CF87BEC43C47551221E40724FE115FF8A4E72D5D46A0E98A934C45CD6904DA0F07499D798EE611497C9493354A9A48C35ECB6318CA55B8322E4295E67F8BC0BE1E0923685E1727B7925920D4F0E9CC30C2A10135DB447EDAD3BCE87C3416252C8B4DF32C24029E0269E7103E80D02DD5A42A99B69A613C6274255DF0599B0DED35A8969463636C6D56D67A05AE11F347A5D5B81896DF5F8A52E6EA7F05359A9FEFC90297BDD298DD77714D3557325DF1C52F42470606ECBFA5E964C0A782AE19CED2E20C73F0438EB597CAE4159B5E5333C97272D8EFEDB49CEB98078E92D990076E6E4101FD97588E4BBAA9DD5D19C671424108EE7FA5F2D74F9F3DEAB4A0AC89CF9833FD9BA1F66719978D7BD13DD2ECDE2BDC9628B1AC1E0A0C44B1408E8869A8B2245DF2A877E01730500AD15466A808E6D9636EEA7A7A0A06568413408E588C52451D189774D84547FBB4171255D6E0BFC9B63C56D582E02FA0F110EEAA2B728E51BC85F529805EBA5E1D6B7323597F1647B0A3DC6D61448C1C062CADE9831DB9E3029322D79D04BB3287B7C5D857AE11802B68921FBC403E390ED693DEAD66E1A728B7F7432408EB2ED9EB9BC3B2BCD8EB2CD44D41A5EBFB32F55BAF47D3AC048F5D1F60B2CB61C0F4E3C178DC7723B8298E9D52771DCF1DABA4088EF74B");
        let x = x % NonZero::new(N2).unwrap();

        assert_eq!(x.as_ring_element(&N2).as_natural_number(), x);
    }

    #[test]
    fn const_log_computes_correctly() {
        assert_eq!(const_log(1), 0);
        assert_eq!(const_log(2), 1);
        assert_eq!(const_log(3), 2);
        assert_eq!(const_log(4), 2);
        assert_eq!(const_log(5), 3);
        assert_eq!(const_log(6), 3);
        assert_eq!(const_log(7), 3);
        assert_eq!(const_log(8), 3);
        assert_eq!(const_log(9), 4);
    }

    #[rstest]
    #[case::n(1)]
    #[case::n(2)]
    #[case::n(5)]
    #[case::n(15)]
    fn n_factorial_is_bounded_correctly(#[case] n: u16) {
        assert!(
            factorial(n) < 2u64.pow(u32::try_from(factorial_upper_bound(usize::from(n))).unwrap())
        )
    }
}

#[cfg(feature = "benchmarking")]
criterion_group!(
    benches,
    proofs::benchmark_proof_of_equality_of_discrete_logs,
    decryption_key_share::benchmark_decryption_share,
    decryption_key_share::benchmark_combine_decryption_shares,
);