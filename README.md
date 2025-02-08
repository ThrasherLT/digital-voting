# Digital Voting System

>Note: This is a pet project I'm doing to familiarize myself with blockchains, but hopefully it could prove useful as a POC for people researching the field of digital elections. Also, the vast majority of this project is still a WIP.

A private and verifiable digital election system based on blind signature cryptography and a custom blockchain.

## Reason

In the first half of the 2020s, the following electoral issues have been gaining the attention of the world's general public:

- Domestic politicians questioning the legitimacy of public election results.
- Militarily aggressive states attempting to influence election results in other countries.

As well as the ever present issue of dictators falsifying democratic elections in an attempt to legitimize themselves.

These issues highlight the need for a more transparent and verifiable election process. Transparancy is easy to achieve using blockchains, but there is almost always a tradeoff between transparancy and privacy, the later being a necessity to avoid political retribution both from political entities as well as members of the general public with overly strong political beliefs.

## Working principle

### Overview

Three parties are present in the process of casting a vote:

- The voter, who casts his vote.
- The election blockchain, which records the voters vote.
- The election authority, which verifies that the voter is elligible for voting.

Blind RSA signatures are used so that the voters activity on the blockchain and with the election authority cannot be correlated.

### Voting Protocol

The votes are cast in the following steps:

- The voter generates a digital signature which he will use to authenticate himself within the blockchain.
- The voter blinds the public key of his digital signature and sends it to a trusted election authority along with his real life credentials.
- The authority verifies that the voter is eligible for voting, signs his blinded public key and sends the signature back to the voter.
- The voter then unblinds the signature he received from the election authority and uses it as an access token to prove his eligibility to vote without exposing any real life personal info to the blockchain.

In this scheme the election authority has no way of correlating the voter's personal data with the voters actual vote on the blockchain.

One major weakness of this scheme is that if the election authority leaks it's private key, a malicious actor might forge limitless votes. To avoid this single point of failure, this system allows requiring multiple access tokens provided by different election authorities.

The actual vote consists of:

- The voter's signature's public key.
- The ID of the candidate for whom the voter is voting for.
- An array of different access tokens each granted by a different election authority.
- A timestamp at which the vote had been created.
- The voters digital signature.

Because all of the access tokens are blind signatures that verify the voter's signature's public key, there is no risk of anyone else stealing the access tokens and using them to cast forged votes.

### Infrastructure

>WIP

### Limitations

Some parts of the digital election process are considered out of scope:

- Voter IP obfuscation is not handled by this blockchain and users are expected to rely on other technologies like a Tor network to hide their IP from the blockchain.
- RSA signatures are not safe from quantum attacks and it is assumed that militarily aggressive states can have access to quantum computers.
- The specific RSA blind signatures crate which is currently used also has a serious vulnerability and should be replaced with a more secure alternative in future versions. The crate is generally decoupled from the rest of the project using wrapper code, so it shouldn't be too complicated to swap out blind signature crates.

## Development

A `docker compose` environment is provided to simulate the different actors of the digital election process and the workings in between them.
A `mock_authority` binary is also being developed alongside this project for assistance with testing.

## Roadmap

> Note: There are still many features missing and issues present before we reach v1.0.0 and this roadmap moreso describes what is considered out of scope for versions < v1.0.0

- [ ] Post quantum and vulnerability-free blind signatures.
- [ ] A way to tie voters and their votes to a specific voting district.
- [ ] Physical voter coercion prevention.
- [ ] Further forged vote detection in case of compromised authority secret keys.
- [ ] Figure out a decent name for this repo.
