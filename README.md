# YTL (Yggdrasil Template Language) v0.3.0
<img src="icon.jpg" width="96" height="96" align="right" />

***YTL*** is a modern, indented configuration language designed specifically for ***Game Development*** and ***Network Engineering.*** It combines the readability of YAML with the power of Nix, offering native data types and a custom, blazing-fast Language Server Protocol (LSP) written in Rust.

# Why YTL?
Traditional configuration formats (JSON, YAML, TOML) were not built for complex systems. They lack native networking types, require boilerplate for environment overrides, and don't understand game development concepts like damage ranges or loot chances.

# YTL solves this by providing:
- ***Realms*** (Environment Overlays): Stop duplicating code for `dev`, `prod`, `Easy`, or `Hardcore`. Use `Realm` to overlay specific changes without rewriting the base block.
- ***ECS Composition (Attach):*** Bring Entity-Component-System architecture to your configs. Use `Attach` to inject traits (like `fire_immunity` or `poison`) directly into entities.
- ***Immutability (Const):*** Protect critical data (like network ports or base stats) from being accidentally overwritten in child blocks.
- ***Native Network Types:*** Out-of-the-box support for IPv4, IPv6, MAC addresses, and CIDR blocks.
- ***Native GameDev Types:*** Built-in support for Vectors `(10, 5, 2)`, RGBA `(255, 0, 0, 255)`, Ranges `13~30` (for damage or loot), and Percentages `25%`.
- ***Smart Rust LSP:*** Includes a custom Language Server written in Rust that provides real-time validation, intelligent hover hints, autocompletion, and interactive field inheritance.


# Syntax Examples
*Game Development (RPG Balance)*

```yaml
Include ytl://base_stats.ytl

Warrior{Character}:
    health 100
    damage 13~30          // Native damage range
    Attach ytl://components/fire_immunity.ytl

    Realm "Hardcore":
        health 50
        damage 25~50      // Overrides only in Hardcore mode

Loot_Goblin:
    Gold 50% 1~5          // 50% chance to drop 1-5 gold
    Sword 15% 1
```
*Network Engineering*
```yaml
Server:
    Const port 8080       // Cannot be modified by child blocks
    ip 192.168.1.1
    subnet 192.168.1.0/24 // Native CIDR

    Realm "prod":
        host "example.com"
        // port 443       <-- This would throw an LSP error because of Const
```
# Tooling & Architecture
YTL is not just a spec; it's a fully-fledged ecosystem:

1.  **TextMate Grammar:** Full syntax highlighting for VS Code, correctly distinguishing between versions (`1.2.3`), IP addresses, ranges, and comments.

2.  **Rust LSP Server (`ytl-lsp`):**
    - **Diagnostics:** Catches syntax errors, invalid IPs/MACs, unclosed brackets, and `Const` violations in real-time.
    - **Interactive Inheritance:** When you inherit a block (`Cax{Car}:`), the LSP offers a QuickPick menu to automatically insert inherited fields.
    - **Hover & Completion:** Context-aware suggestions for `Include`, `Attach`, `Realm`, and block names.

# Installation (Dev Build)
1. Clone the repository.
2. Build the Rust LSP:

```bash
cd ytl-lsp
cargo build --release
```

3. Open the root folder in VS Code and press `F5` to launch the Extension Development Host.

# Roadmap

- [x] **v0.3.0:** New syntax (`Realm`, `Attach`, `Const`), Network validators, TextMate grammar, Rust LSP.
- [ ] **v0.4.0:** C# SDK (for Unity/Godot integration).
- [ ] **v0.5.0:** N/A.
- [ ] **v0.6.0:** N/A.

# License
Apache License 2.0
