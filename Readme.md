# ArcGame: MPC-Native Hidden-Information Game Engine

## 🎮 Overview

**ArcGame** is a next-generation on-chain strategy game infrastructure built on **Arcium** and **Solana**.

Current on-chain games suffer from the "Transparency Paradox"—since all states are public, game mechanics like "Fog of War," "Hidden Hands," or "Secret Moves" are impossible to implement without a centralized dealer. **ArcGame** utilizes **Secure Multi-Party Computation (MPC)** to maintain a hidden game state. Player hands and strategies remain encrypted, while game rules are enforced privately within Arcium's Multi-Party Execution (MXE) environment.

## 🚀 Live Deployment Status (Verified on Devnet v0.8.3)

The engine is fully functional and currently active on the Arcium Devnet.

### 🖥️ Interactive Demo

[Launch ArcGame Terminal](https://silent-builder-x.github.io/ArcGame/)

## 🧠 Core Innovation: Decoupled Logic & Privacy

ArcGame implements a revolutionary gaming primitive:

- **Shielded States:** Player moves and card values are split into **Secret Shares** locally using x25519. No opponent or observer can gain strategic insight from the ledger.
- **Oblivious Battle Resolution:** The Arcis circuit executes logic like `A > B` and `Damage = A - B` entirely within the encrypted domain, revealing only the outcome (e.g., HP reduction) without exposing the inputs.
- **Fair Social Deduction:** Enables zero-trust implementations of games like Mafia, Battleship, or Poker without a trusted third party.

## 🛠 Build & Implementation

```
# Compile Arcis circuits and Solana program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Core Engine:** `resolve_round` (Arcis-MPC Circuit)
- **Security:** Supported by Arcium Multi-Party Execution and Threshold Signatures.
- **Audit Standard:** Built following the **Internal V4** specification with explicit safety documentation.