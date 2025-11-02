# 🧠 Fact Verification Smart Contract (Stellar + Rust)

A **Soroban smart contract** that implements a decentralized **Fact Verification System** — allowing users to submit claims, vote on their authenticity, and retrieve verified results directly on the **Stellar blockchain**.

---

## 🚀 Overview

This repository contains the **backend logic** (smart contract) for a Fact Verification DApp.

The contract allows:
- 📝 Users to **submit new facts**  
- ✅ Other users to **vote** on facts (true/false)  
- 📊 Transparent and **immutable record storage** on-chain  
- 🔍 Retrieval of **all submitted facts** or a **specific fact**  

All votes and submissions are **fully verified on-chain**, and double voting is automatically prevented.

---

## 🧱 Core Contract Functions

| Function | Description |
|-----------|-------------|
| `submit_fact(env, creator, text)` | Submits a new fact and stores it on-chain |
| `vote(env, voter, fact_id, is_true)` | Allows users to vote on a fact (true or false) |
| `get_fact(env, fact_id)` | Fetches details of a single fact |
| `get_all_facts(env)` | Returns a list of all stored facts |
| `get_fact_count(env)` | Returns total number of submitted facts |

---

## ⚙️ Setup Requirements

Make sure you have the following installed:

| Tool | Purpose |
|------|----------|
| [Rust](https://www.rust-lang.org/tools/install) | Programming language for smart contract |
| [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup) | Tool to build, deploy & interact with Stellar contracts |
| [Stellar Testnet Account](https://laboratory.stellar.org/#account-creator?network=test) | Wallet for test deployments |
| [Freighter Wallet](https://freighter.app/) | Browser wallet for Stellar transactions |


