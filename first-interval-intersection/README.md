# Primitive Options

1. Fully Homomorphic Encryption
2. Order-Preserving Encryption (OPE)
3. Order-Revealing Encryption (ORE)

# Methods used

This section will detail the ways in which I have tried to implement interval intersection with PSI.

## Experiment I: Order-Revealing Encryption

Reference used: https://github.com/kevinlewi/fastore/blob/master
ORE is a generalization of OPE => can be built to leak less information

### I.1 Symmetric Encryption

For the first phase of the experiment, we are going to assume that the same key is used by both parties to encrypt and decrypt the interval elements
