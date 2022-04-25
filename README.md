# dagchat

An open source wallet with end-to-end encrypted, on chain messaging for nano and banano using the dagchat protocol.

This application is not yet released. Any versions are test versions and come with no guarantees. Whilst messages are in theory encrypted end-to-end, the encrypted data is being put onto a public blockchain. The cryptography used in the dagchat protocol (and implementation of) has not been audited so it is advised not to include sensitive data within messages. 

# Features
- Import multiple wallets using a mnemonic phrase, hex seed, or even private key. Each wallet supports many accounts which can be shown procedurally or by specifying an index. <br>![image](https://user-images.githubusercontent.com/97409490/165167183-11114b67-71e3-4fcd-85a6-a2a4ff6a0f1e.png) ![image](https://user-images.githubusercontent.com/97409490/165167265-30516a86-5c99-448f-930a-b3ccd1d4bd08.png)
- Send on chain, end to end encrypted memos/messages using the dagchat protocol. <br>![image](https://user-images.githubusercontent.com/97409490/165167726-ec9a9fa9-ffa0-4c2f-8a63-eddc612abdbf.png)
- Receive your nano and banano, and read incoming messages all in the same place. <br>![image](https://user-images.githubusercontent.com/97409490/165168179-358d2fac-57b5-4ef9-b1ec-35db00f3fe2b.png) 
- Messages are identified automatically by the wallet. 
<br>![image](https://user-images.githubusercontent.com/97409490/165168312-18bc63d4-8912-4278-9f83-b2390400ba49.png)
- Messages when sent and received are automatically encrypted and saved to your computer. They can be read again in the message history tab.
<br>![image](https://user-images.githubusercontent.com/97409490/165168937-b41d7884-4dd5-4c60-a934-e57a07b82742.png)

# Building from source
To build dagchat from source, you will need to have rust and cargo installed on your machine: https://www.rust-lang.org/tools/install
1. Clone the repository or download the zip and extract it.
2. If you are building for Linux (Windows and MacOS skip this step) you will need to install some other dependencies that are used for the rust-clipboard crate that manages copying and pasting in dagchat: `sudo apt-get install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`.
3. Enter the repository's directory (either the clone, or the extracted zip) and run `cargo build --release` to build an executable in release mode. This will appear in `/target/release/`.
4. The application should be built and ready to run.
