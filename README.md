# WEWE SOLANA PROGRAM

---

## 🛠️ Deployment on Devnet

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Anchor
[Installation instructions](https://www.anchor-lang.com/docs/installation)

Set the provider to Devnet:

```bash
anchor config set --cluster devnet
anchor config set --provider.wallet ~/.config/solana/id.json
```

### 3. Build and Deploy
```bash
anchor build
anchor deploy
```
After successful deployment, copy the program ID shown in the terminal output.

### 🧪 Test Locally
You can run tests with:

```bash
anchor test
```

### 🔌 User Integration Guide
#### 1. Get the IDL
Once the program is built, the IDL is located in:

```bash
anchor build
target/idl/wewe_token_launch_pad.json
```

#### 2. Using the IDL for integration
```ts
import { AnchorProvider, Program, Idl } from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import idl from "./idl/wewe_token_launch_pad.json";

const connection = new Connection("https://api.devnet.solana.com");
const provider = new AnchorProvider(connection, window.solana, {});
const programId = new PublicKey("<REPLACE_WITH_PROGRAM_ID>");
const program = new Program(idl as Idl, programId, provider);
```
