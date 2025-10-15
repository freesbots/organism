# 🧬 ORGANISM

> **ORGANISM** — a simulation of a self-organizing decentralized network built in **Rust**, inspired by biological and neural systems.  
> Each node is an autonomous agent with its own energy, experience, and behavior.  
> Nodes exchange signals and resources, taking part in economic and evolutionary cycles.  
> The project models **awareness**, **energy exchange**, **evolution**, and **coordination** — aiming to give rise to a *“collective intelligence.”*

---

## 🚀 Overview

**Organism** is an experimental environment where a network of nodes evolves, learns, and redistributes energy based on principles of living systems.  
It combines ideas from biology, neural networks, economics, and decentralized architectures.

---

## 🧩 Architecture

### **1. `main.rs`**
Entry point of the program:
- Launches the network, nodes, economic cycle, and “organism pulse.”  
- Uses **Tokio** for async execution and **Axum** for REST API.  

**Components:**
- Initialize `NetworkBus` — shared communication channel  
- Create nodes (default: 5)  
- Start message handlers  
- Periodic help-signal cycle  
- Evolution cycle (`EnergyEvolution::evolve`)  
- Economy cycle (`EconomyCycle::run`)  
- API server at `127.0.0.1:3000`

---

### **2. `node.rs`**
Defines the `Node` structure:
- `name`
- `energy: Energy { level, node_name }`
- `experience`

**Functionality:**
- Send signals and assist other nodes  
- Participate in economy and evolution  
- Async-safe via `Arc<Mutex<Node>>`

---

### **3. `economy.rs` / `economy_cycle.rs`**
Manages economic balance and resource flow:
- Energy ↔ tokens exchange  
- “Leader fatigue” mechanic  
- Shared `NetworkFund` for redistribution  

---

### **4. `brain.rs`**
Consciousness module:
- Analyzes the state of all nodes  
- Redistributes energy when imbalance > 10 units  
- Forms “conscious” coordinated behavior  
- Future plans: self-learning and memory  

---

### **5. `api.rs`**
REST API built on **Axum**:
- Real-time monitoring of the organism  
- Planned integration with frontend visualization  

---

## 🌱 Current Features

✅ Asynchronous node communication  
✅ Help signals and responses  
✅ Economic cycle (deductions, replenishment, evolution)  
✅ “Leader fatigue” effect  
✅ Basic “consciousness” (`Brain`) and energy redistribution  
✅ Organism “pulse” logic (periodic evolution)  
✅ REST API server  


## ✅ Current Progress

- **Neurocycle (`Brain::run`)** is functional — collects data, processes decisions, and regulates aggressiveness.  
- **Memory module (`Memory`)** stores events and supports data sampling.  
- **Snapshots (`BrainSnapshot`)** synchronize brain state with the API.  
- **API** is accessible and correctly serves `/brain/memory` and `/brain/state`.  
- **Economy and evolution** are operational in their basic form (tokens, energy, fatigue, signals).


---

## 🧠 Next Steps

### 🔹 1. Advanced **Consciousness (`brain.rs`)**
- Add memory and history tracking  
- Weighted decision-making based on past states  
- Priority system (help, growth, accumulation, evolution)  
- Reactions to network events  

### 🔹 2. **Learning / Self-organization**
- Behavior adaptation based on success  
- Simple neural-like influence weights  
- Node roles: leader, empath, strategist  

### 🔹 3. **Blockchain Integration**
- Internal ledger (energy ↔ token transactions)  
- Immutable action records (`Ledger`)  
- Distributed consensus between nodes  

### 🔹 4. **API / Visualization**
- Web UI to visualize the network  
- Graph rendering: nodes, connections, energy flow  
- Adjustable network parameters  

### 🔹 5. **Future Modules**
- `learning.rs` — self-learning & adaptation  
- `ledger.rs` — blockchain and action logs  
- `neuro.rs` — synaptic-style connections  
- `governance.rs` — collective decision-making  

---

## ⚙️ Tech Stack

- 🦀 **Rust**
- ⚙️ **Tokio** — async runtime  
- 🌐 **Axum** — REST API  
- 🎲 **Rand** — randomness and simulation  
- 🔒 **Arc + Mutex** — thread-safe shared state  
- 🪵 **Log / Env_logger** — logging  
- 💾 *(Planned)* **Serde + SQLx** — state persistence  

---

## 🧩 Core Idea

> Build a digital “organism” — a system of autonomous nodes exchanging energy, experience, and tokens,  
> forming coordinated behavior **not hardcoded explicitly**.

---

## 📸 Running the Simulation

```bash
cargo run
```

API available at:
```
http://127.0.0.1:3000
```

---

## 💡 Author

**ORGANISM Project** — an experiment in digital biology and decentralized systems.  
Developed in **Rust**, open-source.

©2025. Kazakov Aleksey

---

### 🧠 “Living systems are not programmed — they learn.”
