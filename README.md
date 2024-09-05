# Crypto

dWallet Labs Ltd. Cryptography Libraries

# Tiresias: Scalable, Maliciously Secure Threshold Paillier

A pure-Rust implementation of the
UC-secure ["Tiresias: Large Scale, Maliciously Secure Threshold Paillier"](https://eprint.iacr.org/2023/998) paper by:

- Offir Friedman (dWallet Labs)
- Avichai Marmor (dWallet Labs)
- Dolev Mutzari (dWallet Labs)
- Yehonatan Cohen Scaly (dWallet Labs)
- Yuval Spiizer (dWallet Labs)
- Avishay Yanai

This is an implementation of the *threshold decryption* protocol only.
For *distributed key generation*, a protocol like
*Diogenes* ([paper](https://eprint.iacr.org/2020/374), [implementation](https://github.com/JustinDrake/LigeroRSA))
should be used.

It is worth mentioning that we also support the *trusted dealer* setting for which one can see examples in our testing &
benchmarking code that uses `secret_sharing/shamir` to deal a secret.

## Security

This implementation relies on [`crypto_bigint`](https://github.com/RustCrypto/crypto-bigint) for constant-time big
integer arithmetics whenever dealing with key material or any other secret information.

We have gone through a rigorous internal auditing process throughout development, requiring the approval of two
additional cryptographers and one additional programmer in every pull request.
That being said, this code has not been audited by a third party yet; use it at your own risk.

## Releases

This code has no official releases yet, and we reserve the right to change some public API until then.

## Performance & Benchmarking

Our code achieves unprecedented scale & performance, with a throughput of about **50 and 3.6 decryptions per _second_**,
when run over a network of **100 and 1000 parties**, respectively.

We have set up an automated GitHub action for benchmarking over an EC2 C6i machine, the result of which could
be [viewed here](https://github.com/odsy-network/tiresias/actions/runs/5363804053/jobs/9731618097).

With the `parallel` feature, we rely on [`rayon`](https://github.com/rayon-rs/rayon) for data parallelism, which, as
shown theoretically in the paper and experimentally, works extremely well in this scheme.

## group

Group traits for abelian groups in additive notation, designed to resemble the cryptographic/mathematics definition as
accurately as possible.
Traits are hierarchical in nature, and higher-level traits embody more specific properties on top of the ones below.
This allows us to capture shared logic between cryptographic groups in the most generic way possible, so that schemes
and protocols could be designed (e.g. [`maurer`](https://github.com/dwallet-labs/maurer)) to work with any group,
including dynamic, unknown order groups like Paillier, and static, prime-order groups like elliptic curves (e.g.,
secp256k1.)

These traits were designed while keeping the security concern of high-level protocols in mind, and as such are
constant-time by default.

Another key addition is `GroupElement::PublicParameters` which captures the relevant information to hash into the
transcript, as required by Fiat-Shamir transforms.
Another important security (and functionality) aspect of the public parameters is the fact they allow us to separate the
group element `GroupElement` from its value `GroupElement::Value`; the former is a runtime representation, which encodes
necessary information for group operations whereas
the latter solely represents the value, which can be serialized and transported over the wire, to later be instantiated
into the former using the group's public parameter `GroupElement::PublicParameters`.
This is important since group operation must always succeed, however, we must also prevent malicious players from
forcing us to use the wrong groups.
For example, if a malicious prover can force the verifier to use a Paillier group for a modulus, they generated
themselves (and thus know how to factor) they can
bypass verification for incorrect claims, or even derive secrets of other parties.
Instead, the verifier should only receive the value of group elements and instantiate the group element using *their
own public parameters*, which ensure operating in the correct group.

## proof

Trait & helpers for zk-proofs and range proofs.

## homomorphic-encryption

Traits for homomorphic encryption schemes, including a threshold homomorphic decryption schemes.

## commitment

Traits for homomorphic commitment schemes, including Pedersen-based implementations.
Includes implementation for hash-based non-homomorphic commitment using [`merlin::Transcript`].

## maurer

Generic Maurer zero-knowledge proofs for any language ${L = {(x, X) | X = \phi(x)}}$ associated with a group
homomorphism
$\phi(x + y) = \phi(x) + \phi(y)$.

## enhanced-maurer

This crate builds upon the `maurer` crate for zero-knowledge proofs over any language $L = {(x, X) | X = \phi(x)}$
associated with a group homomorphism
$\phi(x + y) = \phi(x) + \phi(y)$ and extends it to an _enhanced_ form, in which range claim(s) are a
part of the statement, as defined in Section 4 of the "2PC-MPC: Threshold ECDSA with Thousands of Parties" paper.

# Security

We have gone through a rigorous internal auditing process throughout development, requiring the approval of two
additional cryptographers and one additional programmer in every pull request.
That being said, this code has not been audited by a third party yet; use it at your own risk.

# Releases

This code has no official releases yet, and we reserve the right to change some public API until then.

# Setup & Running

See [Makefile](Makefile)
