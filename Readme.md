# ArcGame: FHE-Native Hidden-Information Game Engine

## 🎮 Overview

**ArcGame** is a next-generation on-chain strategy game infrastructure built on **Arcium** and **Solana**.

Current on-chain games suffer from the "Transparency Paradox"—since all states are public, game mechanics like "Fog of War," "Hidden Hands," or "Secret Moves" are impossible to implement without a centralized dealer. **ArcGame** utilizes **Fully Homomorphic Encryption (FHE)** to maintain a hidden game state. Player hands and strategies remain encrypted, while game rules are enforced privately within Arcium's Multi-Party Execution (MXE) environment.

## 🚀 Live Deployment Status (Verified)

The engine is fully functional and currently active on the Arcium Devnet.

- **MXE Address:** `BhfhWkxAeQSpfD7WQt1GVBovPNDEyfbNSJRS13NYebcd`
- **MXE Program ID:** `GCdPcj1QNKdaGxQWQV3MGxQwaRt65zBRRiN57TGQ21V8`
- **Computation Definition:** `DFXsd1ntxcrnu4cmgvtBxqogojU41SvFJfPha9SDkvJN`
- **Status:** `Active`

## 🧠 Core Innovation: Decoupled Logic & Privacy

ArcGame implements a revolutionary gaming primitive:

- **Shielded States:** Player moves and card values are encrypted locally using x25519. No opponent or observer can gain strategic insight from the ledger.
- **Homomorphic Battle Resolution:** The Arcis circuit executes logic like `A > B` and `Damage = A - B` entirely within the ciphertext space, revealing only the outcome (e.g., HP reduction) without exposing the inputs.
- **Fair Social Deduction:** Enables zero-trust implementations of games like Mafia, Battleship, or Poker without a trusted third party.

## 🛠 Build & Implementation

```
# Compile Arcis circuits and Solana program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Core Engine:** `resolve_round` (Arcis-FHE Circuit)
- **Security:** Supported by Arcium Multi-Party Execution and Threshold Signatures.
- **Audit Standard:** Built following the **Internal V4** specification with explicit safety documentation.