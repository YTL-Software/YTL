# YTL (Yggdrasil Template Language) v0.4.2
<img src="icon.jpg" width="96" height="96" align="right" />

***YTL*** is a modern, indented configuration language designed specifically for ***Game Development*** and ***Network Engineering.*** It combines the readability of YAML with the power of Nix, offering native data types and a custom, blazing-fast Language Server Protocol (LSP) written in Rust.

# Why YTL?
Traditional configuration formats (JSON, YAML, TOML) were not built for complex systems. They lack native networking types, require boilerplate for environment overrides, and don't understand game development concepts like damage ranges or loot chances.

# YTL solves this by providing:
- ***Realms*** (Environment Overlays): Stop duplicating code for `dev`, `prod`, `Easy`, or `Hardcore`. Use `Realm` to overlay specific changes without rewriting the base block.
- ***ECS Composition (Attach):*** Bring Entity-Component-System architecture to your configs. Use `Attach` to inject traits (like `fire_immunity` or `poison`) directly into entities.
- ***Immutability (Const):*** Protect critical data (like network ports or base stats) from being accidentally overwritten in child blocks.
- ***Strict Typing:*** Use the `>` operator for strict validation of complex types (IPv4, IPv6, MAC, UUID, CIDR). Simple types (int, string, bool) remain untyped for maximum readability.
- ***Native Network Types:*** Out-of-the-box support for IPv4, IPv6, MAC addresses, and CIDR blocks.
- ***Native GameDev Types:*** Built-in support for Vectors `(10, 5, 2)`, RGBA `(255, 0, 0, 255)`, Ranges `13~30` (for damage or loot), and Percentages `25%`.
- ***Smart Rust LSP:*** Includes a custom Language Server written in Rust that provides real-time validation, intelligent hover hints, autocompletion, and interactive field inheritance.


# Syntax
For a full guide on syntax, types, and advanced features, please refer to the [Wiki](https://github.com/YTL-Software/YTL-VsCode/wiki/Syntax-Basics).

# Tooling & Architecture
YTL is not just a spec; it's a fully-fledged ecosystem structured as a Cargo Workspace.

1.  **ytl-core:** The foundational parsing engine (Lexer, CST, Validators) independent of the editor.

2.  **ytl-lsp:** The Language Server built on top of `ytl-core`.
    - **Smart Diagnostics:** Catches syntax errors, invalid IPs/MACs, homoglyphs (Unicode visual spoofing), and `Const` violations.
    - **Localization:** Full support for English and Russian with a one-click switch in the status bar.
    - **Interactive Inheritance:** When inheriting a block, the LSP offers a QuickPick menu to automatically insert fields.

3. VS Code Extension: Provides syntax highlighting (TextMate) and integrates the LSP.

# Installation
1. Download the extension from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=Regent.ytl).
2. Or build from source:

```bash
git clone https://github.com/YTL-Software/YTL-VsCode.git
cd YTL-VsCode
cargo build --release
```

Open the root folder in VS Code and press `F5` to launch the Extension Development Host.

# Roadmap
- [x] **v0.4.2:** Cargo Workspace architecture, full localization, homoglyph detection, and stability fixes.
- [ ] **v0.5.0:** CLI compiler— Convert YTL to JSON/YAML/XML and integrate it into Unity/Godot via native C#/C++ bindings
- [ ] **v0.6.0:** N/A.
- [ ] **v0.7.0:** N/A.

# License
Apache License 2.0
