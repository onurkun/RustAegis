# Aegis VM Development Roadmap

> **Version:** 0.1.52 → 0.2.0
> **Son Güncelleme:** 2025-12-05
> **Durum:** Faz 1.2 Tamamlandı ✅

### İlerleme Özeti

```
Faz 1: Hızlı Kazanımlar
├── 1.1 Opaque Predicates     [ ] Bekliyor
└── 1.2 Substitution Fix      [✓] Tamamlandı (2025-12-05)

Faz 2: Core Güçlendirme
├── 2.1 Self-Modifying Code   [ ] Bekliyor
└── 2.2 Handler Mutation      [ ] Bekliyor

Faz 3: Pro Özellikler
├── 3.1 White-box Crypto      [ ] Araştırma
└── 3.2 Advanced CF           [ ] Araştırma
```

---

## Executive Summary

Aegis VM şu anda temel obfuscation yeteneklerine sahip, çalışan bir Rust bytecode VM'i. Bu roadmap, ticari koruma yazılımlarıyla (VMProtect, Themida) rekabet edebilecek seviyeye çıkarmak için gereken geliştirmeleri öncelik sırasına göre tanımlar.

### Mevcut Durum (v0.1.52+)

| Kategori | Durum | Not |
|----------|-------|-----|
| VM Core | ✅ Tamamlandı | 100+ opcode, stack-based |
| Pattern Matching | ✅ Tamamlandı | 45 test, %95 Rust match desteği |
| MBA Transformations | ✅ Aktif | ADD, SUB, XOR için |
| Polymorphic Opcodes | ✅ Aktif | Her build farklı mapping |
| Dead Code Injection | ✅ Aktif | %10 şansla |
| ValueCryptor | ✅ Aktif | Paranoid modda |
| Bytecode Encryption | ✅ Aktif | AES-256-GCM |
| Type Constants | ✅ Aktif | u64::MAX, i64::MIN, BITS, vb. |
| Opaque Predicates | ⚠️ Tanımlı | Kullanılmıyor |
| Complex Substitutions | ✅ **AKTİF** | **Faz 1.2 tamamlandı** |
| Self-Modifying Code | ❌ Yok | |
| Handler Mutation | ❌ Yok | |
| Anti-Debug | ❌ Yok | Ayrı ürün olacak |

---

## Faz 1: Hızlı Kazanımlar

> **Süre:** 1-2 gün
> **Hedef Versiyon:** 0.1.53
> **Lisans:** MIT (Core)

### 1.1 Opaque Predicates Aktivasyonu

**Durum:** Kod hazır, entegrasyon gerekli
**Efor:** ~2 saat
**Etki:** Orta-Yüksek

#### Açıklama

Opaque predicates, her zaman true veya false dönen ama statik analizle belirlenemeyen koşullardır. Kod `substitution.rs:757-886`'da zaten tanımlı, sadece compiler'dan çağrılması gerekiyor.

#### Mevcut Kod

```rust
// substitution.rs - MEVCUT
pub struct OpaquePredicate;

impl OpaquePredicate {
    /// Stack: [] -> [1] (always true)
    pub fn emit_always_true<F: Fn(u8) -> u8>(...) { ... }

    /// Stack: [] -> [0] (always false)
    pub fn emit_always_false<F: Fn(u8) -> u8>(...) { ... }
}

pub struct ControlFlowSubstitution;

impl ControlFlowSubstitution {
    /// Fake conditional that always falls through
    pub fn emit_fake_conditional<F: Fn(u8) -> u8>(...) { ... }
}
```

#### Yapılacaklar

- [ ] `emit.rs` → `emit_jump()` içine opaque predicate injection ekle
- [ ] `emit.rs` → `emit_op()` içine rastgele fake conditional ekle
- [ ] Yeni flag: `opaque_predicates_enabled` (standard+ için true)
- [ ] Test: Opaque predicate'li bytecode'un doğru çalıştığını doğrula

#### Implementasyon

```rust
// emit.rs - EKLENECEK
pub(crate) fn emit_jump(&mut self, opcode: u8, label: &str) {
    // Opaque predicate injection (%15 şans)
    if self.opaque_enabled && self.should_inject_opaque() {
        let table = self.opcode_table.clone();
        let encode = |op: u8| table.encode(op);
        ControlFlowSubstitution::emit_fake_conditional(
            &mut self.subst,
            &mut self.bytecode,
            &encode,
        );
    }

    self.emit_op(opcode);
    let fixup_pos = self.pos();
    self.emit_u16(0);
    self.fixups.push((fixup_pos, label.to_string()));
}

fn should_inject_opaque(&mut self) -> bool {
    let entropy = self.bytecode.len() as u64 * 0x9E3779B97F4A7C15;
    (entropy % 100) < 15
}
```

---

### 1.2 Disabled Substitutions Fix ✅ TAMAMLANDI

**Durum:** ✅ Tamamlandı (2025-12-05)
**Efor:** ~2 saat (tahminden hızlı)
**Etki:** Orta-Yüksek

#### Açıklama

`AddSubstitution` ve `SubSubstitution`'ın complex varyantları "stack order issues" nedeniyle disabled idi. Matematiksel doğrulama sonrası tüm varyantların doğru çalıştığı tespit edildi ve aktif edildi.

#### Yapılan Değişiklikler

**1. Type Constants Desteği (Bonus)**
`aegis_vm_macro/src/compiler/expr.rs` dosyasına eklendi:
- `u64::MAX`, `u64::MIN`, `i64::MAX`, `i64::MIN`
- `u32::MAX`, `u16::MAX`, `u8::MAX` ve signed karşılıkları
- `BITS` sabitleri (u8::BITS, u64::BITS, vb.)

**2. AddSubstitution Aktivasyonu**
`aegis_vm_macro/src/substitution.rs:360-368`:
```rust
// ÖNCE (disabled):
match subst.next_rand() % 5 {
    // 0 => Self::SubNegate,   // Temporarily disabled
    // 1 => Self::NotSubNot,   // Temporarily disabled
    _ => Self::Original,
}

// SONRA (aktif):
match subst.next_rand() % 5 {
    0 => Self::SubNegate,   // a + b = a - (-b)
    1 => Self::NotSubNot,   // a + b = ~(~a - b)
    _ => Self::Original,
}
```

**3. SubSubstitution Aktivasyonu**
`aegis_vm_macro/src/substitution.rs:430-438`:
```rust
// ÖNCE (disabled):
match subst.next_rand() % 5 {
    // 0 => Self::AddNegate,   // Temporarily disabled
    // 1 => Self::NotAddNot,   // Temporarily disabled
    _ => Self::Original,
}

// SONRA (aktif):
match subst.next_rand() % 5 {
    0 => Self::AddNegate,   // a - b = a + (-b)
    1 => Self::NotAddNot,   // a - b = ~(~a + b)
    _ => Self::Original,
}
```

#### Tamamlanan Görevler

- [x] `AddSubstitution::SubNegate` stack order doğrulandı ve aktif edildi
- [x] `AddSubstitution::NotSubNot` stack order doğrulandı ve aktif edildi
- [x] `SubSubstitution::AddNegate` stack order doğrulandı ve aktif edildi
- [x] `SubSubstitution::NotAddNot` stack order doğrulandı ve aktif edildi
- [x] `XorSubstitution` zaten aktifti (OrAndNot, MaskedOr)
- [x] 24 substitution testi oluşturuldu (`tests/substitution_test.rs`)
- [x] **650 test geçti** - tüm protection level'ları doğru çalışıyor

#### Matematiksel Doğrulama

| Varyant | Formül | Stack Operasyonları |
|---------|--------|---------------------|
| SubNegate | a + b = a - (-b) | NOT, INC, SUB |
| NotSubNot | a + b = ~(~a - b) | SWAP, NOT, SWAP, SUB, NOT |
| AddNegate | a - b = a + (-b) | NOT, INC, ADD |
| NotAddNot | a - b = ~(~a + b) | SWAP, NOT, SWAP, ADD, NOT |

---

## Faz 2: Core Güçlendirme

> **Süre:** 1 hafta
> **Hedef Versiyon:** 0.2.0
> **Lisans:** MIT (Core)

### 2.1 Self-Modifying Code (SMC)

**Durum:** Tasarım aşaması
**Efor:** ~3 gün
**Etki:** Çok Yüksek

#### Açıklama

Bytecode runtime'da şifreli kalır, sadece çalıştırılacak instruction decrypt edilir, çalıştırıldıktan sonra tekrar encrypt edilir. Bu, memory dump'ı işe yaramaz hale getirir.

#### Tasarım

```
┌─────────────────────────────────────────────────────────────────┐
│                    SMC EXECUTION MODEL                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  T=0 (Başlangıç):                                               │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐              │
│  │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │  Tümü şifreli │
│  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘              │
│                                                                 │
│  T=1 (IP=0):                                                    │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐              │
│  │ ADD │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │  Sadece [0]   │
│  └──▲──┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘   açık       │
│     └── execute & re-encrypt                                    │
│                                                                 │
│  T=2 (IP=1):                                                    │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐              │
│  │ ??? │ SUB │ ??? │ ??? │ ??? │ ??? │ ??? │ ??? │  [0] tekrar   │
│  └─────┴──▲──┴─────┴─────┴─────┴─────┴─────┴─────┘   şifreli    │
│           └── execute & re-encrypt                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### API Tasarımı

```rust
// engine.rs - YENİ
pub struct SmcConfig {
    /// SMC aktif mi?
    pub enabled: bool,
    /// Aynı anda kaç opcode açık olabilir (sliding window)
    pub window_size: usize,
    /// XOR key (runtime'da türetilir)
    pub key: [u8; 32],
}

impl Default for SmcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            window_size: 1,
            key: [0; 32],
        }
    }
}

pub fn execute_smc(
    bytecode: &[u8],
    input: &[u8],
    config: SmcConfig,
) -> Result<u64, VmError> {
    let mut code = bytecode.to_vec();
    let mut state = VmState::new(input);
    let mut decrypted_window: VecDeque<usize> = VecDeque::new();

    while state.running {
        let ip = state.ip as usize;

        // Decrypt current instruction
        decrypt_at(&mut code, ip, &config.key);
        decrypted_window.push_back(ip);

        // Re-encrypt old instructions (outside window)
        while decrypted_window.len() > config.window_size {
            let old_ip = decrypted_window.pop_front().unwrap();
            encrypt_at(&mut code, old_ip, &config.key);
        }

        // Execute
        execute_single(&mut state, &code)?;
    }

    Ok(state.result())
}
```

#### Yapılacaklar

- [ ] `SmcConfig` struct tanımla
- [ ] `execute_smc()` fonksiyonu implement et
- [ ] Sliding window mekanizması
- [ ] Jump target handling (decrypt before jump)
- [ ] Multi-byte instruction handling
- [ ] Performance benchmark
- [ ] Test suite

#### Zorluklar ve Çözümler

| Zorluk | Çözüm |
|--------|-------|
| Jump target şifreli | Jump öncesi lookahead decrypt |
| Multi-byte instructions | Instruction boundary tracking |
| Performance | Window size ayarlanabilir |
| WASM uyumsuzluk | Native-only feature flag |

---

### 2.2 Handler Mutation

**Durum:** Tasarım aşaması
**Efor:** ~4 gün
**Etki:** Çok Yüksek
**Lisans:** Pro (Ücretli)

#### Açıklama

Her build'de handler kodları farklı olacak. Aynı işlemi yapan ama farklı instruction sequence'leri kullanan handler'lar generate edilecek.

#### Mevcut Durum

```rust
// engine.rs - ŞU AN (statik)
fn handle_add(state: &mut VmState) {
    let b = state.stack.pop();
    let a = state.stack.pop();
    state.stack.push(a.wrapping_add(b));
}
```

#### Hedef

```rust
// engine.rs - MUTATED (her build farklı)

// Build A:
fn handle_add(state: &mut VmState) {
    let _junk1 = 0x12345678_u64;  // junk
    let b = state.stack.pop();
    let _junk2 = _junk1 ^ 0xDEADBEEF;  // junk
    let a = state.stack.pop();
    state.stack.push(a.wrapping_add(b));
    let _junk3 = state.stack.len();  // junk
}

// Build B:
fn handle_add(state: &mut VmState) {
    let b = state.stack.pop();
    let a = state.stack.pop();
    // Equivalent: a + b = (a ^ b) + 2 * (a & b)
    let xor_result = a ^ b;
    let and_result = a & b;
    let doubled = and_result << 1;
    state.stack.push(xor_result.wrapping_add(doubled));
}

// Build C:
fn handle_add(state: &mut VmState) {
    let b = state.stack.pop();
    let a = state.stack.pop();
    // Equivalent: a + b = a - (-b)
    let neg_b = (!b).wrapping_add(1);
    state.stack.push(a.wrapping_sub(neg_b));
}
```

#### Implementasyon Stratejisi

```rust
// build.rs - Handler generation
fn generate_mutated_handlers(seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let mut output = String::new();

    for (name, base_impl) in HANDLERS {
        let mutation = select_mutation(&mut rng, name);
        let junk_count = rng.next() % 5;

        output += &format!(
            "fn handle_{}(state: &mut VmState) {{\n",
            name
        );

        // Insert junk before
        for _ in 0..junk_count/2 {
            output += &generate_junk_line(&mut rng);
        }

        // Mutated implementation
        output += &apply_mutation(base_impl, mutation);

        // Insert junk after
        for _ in 0..junk_count/2 {
            output += &generate_junk_line(&mut rng);
        }

        output += "}\n\n";
    }

    output
}
```

#### Yapılacaklar

- [ ] Handler template sistemi tasarla
- [ ] Mutation varyantları tanımla (her opcode için 3-5 varyant)
- [ ] Junk code generator
- [ ] `build.rs` entegrasyonu
- [ ] Generated code'un doğruluğunu test et
- [ ] Benchmark: Binary size vs mutation level

#### Mutation Varyantları

| Opcode | Varyant 1 | Varyant 2 | Varyant 3 |
|--------|-----------|-----------|-----------|
| ADD | Direct | XOR+AND*2 | SUB(-b) |
| SUB | Direct | ADD(-b) | NOT+ADD+NOT |
| XOR | Direct | OR-AND | (a&~b)\|(~a&b) |
| AND | Direct | ~(~a\|~b) | De Morgan |
| OR | Direct | ~(~a&~b) | De Morgan |
| NOT | Direct | XOR(MAX) | SUB from MAX |

---

## Faz 3: Pro Özellikler

> **Süre:** 2+ hafta
> **Hedef Versiyon:** 0.3.0
> **Lisans:** Pro (Ücretli)

### 3.1 White-box Cryptography

**Durum:** Araştırma aşaması
**Efor:** ~2 hafta
**Etki:** Orta

#### Açıklama

AES key'i lookup table'lara gömerek, key'in memory'den extract edilmesini zorlaştır.

#### Not

Bu özellik karmaşık ve araştırma gerektirir. Detaylı tasarım Faz 3 başlangıcında yapılacak.

---

### 3.2 Advanced Control Flow

**Durum:** Araştırma aşaması
**Efor:** ~1 hafta
**Etki:** Orta

#### Açıklama

- Computed jumps (jump target runtime'da hesaplanır)
- Handler chaining (bir handler diğerini çağırır)
- Indirect dispatch (switch yerine function pointer table)

---

## Ayrı Ürün: Enterprise Edition

> **Lisans:** Ticari
> **Not:** Bu özellikler ayrı bir repo/ürün olarak geliştirilecek

### Anti-Debug Suite

- IsDebuggerPresent / CheckRemoteDebuggerPresent
- NtQueryInformationProcess
- PEB.BeingDebugged / NtGlobalFlag
- Timing checks (RDTSC, QueryPerformanceCounter)
- Hardware breakpoint detection
- INT3 scanning
- Debug register checks

### Anti-VM Detection

- VMware/VirtualBox/QEMU detection
- CPUID checks
- Registry artifacts
- MAC address patterns

### Runtime Integrity

- Continuous hash verification
- Code section monitoring
- Hook detection

---

## Timeline

```
┌─────────────────────────────────────────────────────────────────┐
│  2025 Q1                                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Aralık W1:    Faz 1.2 (Substitution Fix) ✅ TAMAMLANDI        │
│                └─ Type constants, 4 substitution aktif          │
│                └─ 650 test geçti                                │
│                                                                 │
│  Aralık W1-2:  Faz 1.1 (Opaque Predicates)                     │
│                └─ v0.1.53 Release                               │
│                                                                 │
│  Aralık W3-4:  Faz 2.1 (Self-Modifying Code)                   │
│                                                                 │
│  Ocak W1-2:    Faz 2.2 (Handler Mutation)                      │
│                └─ v0.2.0 Release                                │
│                                                                 │
│  Ocak W3-4:    Faz 3 Başlangıç                                 │
│                                                                 │
│  Şubat:        Pro Edition Beta                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Success Metrics

| Metrik | Hedef | Mevcut |
|--------|-------|--------|
| Test Count | 500+ | **650** ✅ |
| Test Coverage | >90% | ~85% |
| Binary Size Increase | <50% | ~30% |
| Performance Overhead | <5x | ~3x (debug), ~8x (paranoid) |
| Supported Rust Patterns | %98 | ~95% |
| RE Time (orta seviye) | >1 hafta | TBD |

---

## Risk ve Mitigation

| Risk | Olasılık | Etki | Mitigation | Durum |
|------|----------|------|------------|-------|
| SMC WASM uyumsuz | Yüksek | Düşük | Native-only flag | Bekliyor |
| Handler mutation binary bloat | Orta | Orta | Configurable mutation level | Bekliyor |
| Stack order bugs | ~~Düşük~~ | ~~Yüksek~~ | ~~Extensive testing~~ | ✅ Çözüldü |
| Performance regression | Orta | Orta | Benchmark suite | Bekliyor |

---

## Appendix: Dosya Değişiklikleri

### Faz 1.2 ✅ (Tamamlandı)

```
aegis_vm_macro/src/
├── compiler/
│   └── expr.rs          [MODIFIED] Type constants (u64::MAX, etc.)
└── substitution.rs      [MODIFIED] Enabled AddSubstitution & SubSubstitution

aegis_vm/tests/
└── substitution_test.rs [NEW] 24 substitution correctness tests
```

### Faz 1.1 (Bekliyor)

```
aegis_vm_macro/src/
├── compiler/
│   └── emit.rs          [TODO] Opaque predicate injection
```

### Faz 2

```
aegis_vm/src/
├── engine.rs            [MODIFY] SMC execution mode
├── smc.rs               [NEW] Self-modifying code logic
└── build.rs             [MODIFY] Handler mutation generation

aegis_vm_macro/src/
└── lib.rs               [MODIFY] SMC flag propagation
```

---

*Bu döküman yaşayan bir döküman olup, geliştirme sürecinde güncellenecektir.*
