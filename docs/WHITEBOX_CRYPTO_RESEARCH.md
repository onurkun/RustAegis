# White-Box Cryptography Araştırma Raporu

> **Tarih:** 2025-12-05
> **Durum:** Araştırma Tamamlandı
> **Hedef:** Aegis VM entegrasyonu için fizibilite analizi

---

## Executive Summary

White-box cryptography (WBC), şifreleme anahtarını "görünmez" hale getirmek için tasarlanmış bir tekniktir. Saldırganın tüm koda ve belleğe tam erişimi olduğu varsayılır (white-box modeli). Anahtar, lookup table'lara gömülerek gizlenir.

### Kritik Bulgular

| Konu | Durum | Not |
|------|-------|-----|
| Güvenlik | ⚠️ **KIRIK** | Tüm bilinen WBC şemaları kırılmış durumda |
| BGE Saldırısı | 2^22 - 2^30 | Chow şeması için key extraction |
| DCA Saldırısı | Pratik | Nibble encoding'li tüm implementasyonlar kırılabilir |
| Tablo Boyutu | ~500 KB | AES-128 için (Chow şeması) |
| Performans | ~100-1000x | Normal AES'e göre yavaşlama |

**Sonuç:** White-box AES, tek başına güvenli değil. Ancak diğer koruma teknikleriyle birlikte (SMC, handler mutation, opaque predicates) katmanlı savunma sağlayabilir.

---

## 1. Temel Kavramlar

### 1.1 White-Box Model Nedir?

```
┌─────────────────────────────────────────────────────────────┐
│                    SALDIRGAN MODELLERİ                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Black-Box:    Saldırgan sadece input/output görür          │
│  ┌───────┐                                                  │
│  │ ?AES? │──▶ Ciphertext                                    │
│  └───────┘                                                  │
│                                                             │
│  Grey-Box:    Saldırgan yan kanal (power, timing) ölçer     │
│  ┌───────┐    ~~~~ power trace ~~~~                        │
│  │  AES  │──▶ Ciphertext                                    │
│  └───────┘                                                  │
│                                                             │
│  White-Box:   Saldırgan HER ŞEYE erişebilir                 │
│  ┌───────────────────────────────────────┐                  │
│  │  for i in 0..10:                      │                  │
│  │    state = sbox[state ^ round_key[i]] │ ← Kod görünür    │
│  │    state = mix_columns(state)         │                  │
│  │  return state                         │                  │
│  └───────────────────────────────────────┘                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Neden Gerekli?

- **DRM:** Video/müzik şifre çözme anahtarları kullanıcı cihazında
- **Mobile Payments:** Ödeme anahtarları güvenilmeyen ortamda
- **Malware Ortamı:** Keylogger/debugger aktifken bile güvenlik
- **Lisans Doğrulama:** Lisans anahtarı binary'de gömülü

---

## 2. Chow et al. Şeması (2002)

### 2.1 Temel Fikir

AES'i lookup table'lar ağına dönüştür:

```
Normal AES:                      White-Box AES:
┌─────────────┐                  ┌─────────────┐
│ SubBytes    │                  │  T-Box[i]   │ ← Key + SubBytes birleşik
│ ShiftRows   │        ──▶       │  Ty-Box[i]  │ ← MixColumns dahil
│ MixColumns  │                  │  XOR Tables │ ← XOR işlemi table lookup
│ AddRoundKey │                  │  MB × L     │ ← Mixing bijections
└─────────────┘                  └─────────────┘
```

### 2.2 Tablo Yapısı

```c
// T-Box: SubBytes + AddRoundKey birleşik
// Input: 8-bit, Output: 8-bit
uint8_t Tbox[10][16][256];  // 10 round × 16 byte × 256 değer

// Ty-Box: T-Box + MixColumns birleşik
// Input: 8-bit, Output: 32-bit
uint32_t Tybox[9][16][256];  // 9 round × 16 byte × 256 değer

// XOR Tables: XOR işlemini table lookup'a dönüştür
// Input: 2×4-bit, Output: 4-bit
uint8_t Xor[9][96][16][16];  // 4-bit XOR tables

// MBL: inv(MixingBijection) × L encoding
uint32_t MBL[9][16][256];
```

### 2.3 Tablo Boyutları

| Tablo | AES-128 | AES-256 |
|-------|---------|---------|
| XOR tables | 221 KB | 319 KB |
| Tyi-boxes | 147 KB | 213 KB |
| T-boxes (son round) | 4 KB | 4 KB |
| inv(MB) × L | 147 KB | 213 KB |
| **TOPLAM** | **520 KB** | **750 KB** |

### 2.4 Koruma Katmanları

```
┌─────────────────────────────────────────────────────────────┐
│                    CHOW ŞEMASI KORUMALARI                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. ENCODING (f, g bijections)                              │
│     ┌───────┐                                               │
│     │ Table │ = g ∘ T ∘ f⁻¹                                │
│     └───────┘                                               │
│     Orijinal değerler yerine encoded değerler saklanır      │
│                                                             │
│  2. MIXING BIJECTIONS (MB)                                  │
│     32×32 veya 128×128 invertible matrix                    │
│     Intermediate değerleri karıştırır                       │
│                                                             │
│  3. EXTERNAL ENCODINGS                                      │
│     Input/output'a uygulanan encoding                       │
│     Caller'ın decode etmesi gerekir                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Bilinen Saldırılar

### 3.1 BGE Attack (Billet-Gilbert-Ech-Chatbi, 2004)

**Karmaşıklık:** 2^22 - 2^30

```
Saldırı Adımları:
1. T-box'lardan affine equivalence bul
2. MixColumns'un lineer yapısını kullan
3. Round key'leri bir bir recover et
4. Master key'i türet
```

**Araçlar:**
- [Blue Galaxy Energy](https://blog.quarkslab.com/blue-galaxy-energy-a-new-white-box-cryptanalysis-open-source-tool.html) - BGE attack implementation
- [ph4r05/Whitebox-crypto-AES](https://github.com/ph4r05/Whitebox-crypto-AES) - BGE attack + generator

### 3.2 DCA Attack (Differential Computation Analysis)

**Özellik:** Reverse engineering gerektirmez!

```
┌─────────────────────────────────────────────────────────────┐
│                      DCA SALDIRISI                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Programı çalıştır, computation trace kaydet             │
│     (memory access, register values, vb.)                   │
│                                                             │
│  2. Farklı input'larla tekrarla                             │
│                                                             │
│  3. Trace'lere DPA (Differential Power Analysis) uygula     │
│     - Korelasyon analizi                                    │
│     - Intermediate value tahminleri                         │
│                                                             │
│  4. Key recover                                             │
│                                                             │
│  NOT: 4-bit (nibble) encoding kullanan TÜM implementasyonlar│
│       DCA'ya karşı savunmasız!                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Araçlar:**
- [SideChannelMarvels/Deadpool](https://github.com/SideChannelMarvels/Deadpool) - DCA attack framework

### 3.3 WhibOx Yarışmaları

| Yıl | Sonuç |
|-----|-------|
| 2017 | Tüm implementasyonlar kırıldı |
| 2019 | Tüm implementasyonlar kırıldı |
| 2021 | Tüm implementasyonlar kırıldı |
| 2024 | 47 submission, TÜMÜ kırıldı (en güçlü olan ~5 gün dayandı) |

---

## 4. Modern Yaklaşımlar ve Alternatifler

### 4.1 Karroumi Şeması (2011)

Dual AES kullanır - farklı generating polynomial ile ikinci bir AES.

**Durum:** BGE attack tarafından da kırıldı.

### 4.2 WB-VLUT (2024)

- Time-bound versioned lookup tables
- Ephemeral key generation
- Zero key retention in memory

**Durum:** Henüz yaygın saldırı yok, ancak yeni.

### 4.3 Masking + Shuffling

- DCA'ya karşı boolean masking
- Tablo erişim sırasını rastgele yap

**Durum:** Overhead çok yüksek, pratik değil.

---

## 5. Aegis VM Entegrasyon Analizi

### 5.0 Uygulama Kılavuzu (Kritik Kararlar)

#### 5.0.1 Performance/Security Trade-off

```
┌─────────────────────────────────────────────────────────────┐
│           PERFORMANS/GÜVENLİK DENGESİ KARARI                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⚠️  WBC 100-1000x yavaş! Nerede kullanılacağı KRİTİK.      │
│                                                             │
│  ✅ KULLAN:                                                 │
│     • Startup (bytecode decryption) - Tek seferlik          │
│     • Key derivation - Başlangıçta bir kez                  │
│     • License validation - Nadiren çağrılır                 │
│                                                             │
│  ❌ KULLANMA:                                               │
│     • VM execution loop - Her opcode'da çağrılır!           │
│     • SMC decrypt/encrypt - Her instruction'da!             │
│     • Hot path'ler - Performance killer                     │
│                                                             │
│  SONUÇ: WBC sadece "cold path" (startup) için.              │
│         Runtime encryption için lightweight XOR/ChaCha.     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Önerilen Mimari:**
```
┌──────────────────────────────────────────────────────────┐
│  STARTUP PHASE (WBC ile)                                 │
│  ┌────────────────────────────────────────────────────┐  │
│  │  1. WBC_decrypt(encrypted_bytecode) → bytecode     │  │
│  │  2. WBC_derive(BUILD_SEED) → smc_key               │  │
│  │  3. Memory'de smc_key sakla (obfuscated)           │  │
│  └────────────────────────────────────────────────────┘  │
│                          ↓                               │
│  RUNTIME PHASE (Lightweight crypto ile)                  │
│  ┌────────────────────────────────────────────────────┐  │
│  │  4. SMC: XOR(instruction, smc_key) - Hızlı!        │  │
│  │  5. Execute instruction                            │  │
│  │  6. Re-XOR → tekrar encrypt                        │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

#### 5.0.2 Key Derivation (Anahtar Türetimi)

```
┌─────────────────────────────────────────────────────────────┐
│              WBC TABLOLARI NASIL ÜRETİLECEK?                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Kaynak: BUILD_SEED (zaten build.rs'de var)                 │
│                                                             │
│  // build.rs                                                │
│  let build_seed = generate_build_seed();  // 32 bytes       │
│                                                             │
│  // WBC key derivation                                      │
│  let wbc_key = hmac_sha256(&build_seed, b"whitebox-aes");   │
│  let wbc_tables = generate_chow_tables(&wbc_key[0..16]);    │
│                                                             │
│  NEDEN BUILD_SEED?                                          │
│  • Zaten her build'de farklı                                │
│  • Entropy pool'dan türetiliyor                             │
│  • Diğer obfuscation'larla senkronize                       │
│                                                             │
│  DETERMİNİZM:                                               │
│  • Aynı BUILD_SEED → Aynı tablolar                          │
│  • Reproducible builds için gerekli                         │
│  • Test edilebilirlik                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 5.0.3 Rust/WASM Optimizasyonu

```
┌─────────────────────────────────────────────────────────────┐
│              PLATFORM-SPESİFİK KARARLAR                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  SORUN: 500KB tablo WASM için büyük!                        │
│  • WASM binary size kritik (CDN, mobile)                    │
│  • WASM'da memory protection yok zaten                      │
│                                                             │
│  ÇÖZÜM: Feature flag arkasına al                            │
│                                                             │
│  // Cargo.toml                                              │
│  [features]                                                 │
│  default = ["std", "handler_mutation"]                      │
│  whitebox = []           # Opt-in, default kapalı           │
│  whitebox_full = []      # Tüm tablolar (500KB)             │
│  whitebox_lite = []      # Sadece T-box (50KB)              │
│                                                             │
│  // lib.rs                                                  │
│  #[cfg(feature = "whitebox")]                               │
│  pub mod whitebox;                                          │
│                                                             │
│  PLATFORM MATRİSİ:                                          │
│  ┌─────────────┬───────────┬───────────┐                   │
│  │ Platform    │ whitebox  │ Neden?    │                   │
│  ├─────────────┼───────────┼───────────┤                   │
│  │ Native x86  │ ✅ Açık   │ RAM bol   │                   │
│  │ Native ARM  │ ✅ Açık   │ Mobile OK │                   │
│  │ WASM        │ ❌ Kapalı │ Size!     │                   │
│  │ Embedded    │ ❌ Kapalı │ RAM yok   │                   │
│  └─────────────┴───────────┴───────────┘                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

### 5.1 Nerede Kullanılabilir?

```
┌─────────────────────────────────────────────────────────────┐
│              AEGIS VM WHITEBOX KULLANIM ALANLARI             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. BYTECODE ENCRYPTION KEY                                 │
│     Şu an: AES-256-GCM key binary'de plaintext              │
│     WBC ile: Key lookup table'lara gömülü                   │
│                                                             │
│  2. SMC ENCRYPTION KEY                                      │
│     Self-modifying code'un XOR key'i                        │
│     WBC ile: Runtime'da key "görünmez"                      │
│                                                             │
│  3. BUILD_SEED KORUASI                                      │
│     Opcode shuffle için kullanılan seed                     │
│     WBC ile: Seed extraction zorlaştırılır                  │
│                                                             │
│  4. WATERMARK VERIFICATION                                  │
│     Müşteri ID doğrulama                                    │
│     WBC ile: Verification key gizli                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Entegrasyon Seçenekleri

#### Seçenek A: Build-Time Table Generation

```rust
// build.rs
fn generate_whitebox_tables(key: &[u8; 16]) {
    // Chow şeması tablo üretimi
    let tyboxes = generate_tyboxes(key);
    let xor_tables = generate_xor_tables();
    let mbl_tables = generate_mbl_tables();

    // Rust source olarak yaz
    write_tables_to_rust("whitebox_tables.rs", &tyboxes, &xor_tables, &mbl_tables);
}
```

**Avantajlar:**
- Her build'de farklı tablolar
- Key binary'de hiç görünmez
- Mevcut build.rs altyapısına uyumlu

**Dezavantajlar:**
- ~500 KB binary size artışı
- Tablolar statik analiz edilebilir

#### Seçenek B: Runtime Table Derivation

```rust
// Tablolar runtime'da türetilir
fn derive_whitebox_tables(encoded_seed: &[u8]) -> WhiteboxTables {
    // Seed'den key türet
    let key = derive_key_from_seed(encoded_seed);
    // Tabloları hesapla
    generate_tables(&key)
}
```

**Avantajlar:**
- Tablolar binary'de yok
- Daha küçük binary

**Dezavantajlar:**
- Startup overhead (~100ms)
- derive_key_from_seed fonksiyonu analiz edilebilir

#### Seçenek C: Hybrid (Önerilen)

```rust
// 1. Build-time'da "partial" tablolar üret
// 2. Runtime'da SMC key ile XOR'la
// 3. Tablolar sadece kullanılırken "açık"

struct HybridWhitebox {
    encrypted_tables: Vec<u8>,  // Build-time generated, encrypted

    fn decrypt_on_demand(&mut self, round: usize) {
        // SMC benzeri: sadece gerekli round'u decrypt
        decrypt_round_tables(round);
    }
}
```

### 5.3 Mevcut Korumalarla Kombinasyon

```
┌─────────────────────────────────────────────────────────────┐
│              KATMANLI SAVUNMA STRATEJİSİ                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Layer 5: Anti-Debug / Anti-VM (Enterprise)                 │
│           ↓                                                 │
│  Layer 4: WHITEBOX CRYPTO ← NEW!                            │
│           Key'ler lookup table'larda                        │
│           ↓                                                 │
│  Layer 3: Self-Modifying Code (SMC)                         │
│           WBC tabloları da SMC ile korunur                  │
│           ↓                                                 │
│  Layer 2: Handler Mutation + Opaque Predicates              │
│           WBC lookup fonksiyonları da mutate edilir         │
│           ↓                                                 │
│  Layer 1: Polymorphic Opcodes + MBA                         │
│           Her build farklı                                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.4 Tahmini Efor ve ROI

| Görev | Efor | Etki |
|-------|------|------|
| Rust'ta Chow şeması implement et | 2-3 hafta | Orta |
| Build.rs entegrasyonu | 3-5 gün | Yüksek |
| SMC ile kombinasyon | 1 hafta | Çok Yüksek |
| DCA karşı masking | 2+ hafta | Belirsiz |

**ROI Değerlendirmesi:**
- Tek başına WBC: Düşük (kırılabilir)
- SMC + Handler Mutation ile: Orta-Yüksek (katmanlı savunma)
- Tüm korumalarla: Yüksek (RE süresini önemli ölçüde artırır)

---

## 6. Implementasyon Referansları

### 6.1 Analiz Edilen Repolar

| Repo | Dil | Özellikler | Lisans |
|------|-----|------------|--------|
| [balena/aes-whitebox](https://github.com/balena/aes-whitebox) | C++ | Chow şeması, L×MB, AES-128/192/256 | BSD |
| [ph4r05/Whitebox-crypto-AES](https://github.com/ph4r05/Whitebox-crypto-AES) | C++ | Chow + Karroumi + BGE Attack | GPL |
| [chrku/whitebox_crypto](https://github.com/chrku/whitebox_crypto) | C++ | Header generator | - |

### 6.2 balena/aes-whitebox Analizi

**Dosya Yapısı:**
```
aes_whitebox_compiler.cc  → Tablo üretici
aes_whitebox.cc           → Runtime cipher
aes_whitebox_tables.cc    → Generated tables (520 KB)
```

**Tablo Üretimi:**
```cpp
void CalculateTyBoxes(uint32_t roundKey[],
    uint32_t Tyboxes[][16][256],     // T × MixCol
    uint8_t TboxesLast[16][256],     // Son round
    uint32_t MBL[][16][256],         // inv(MB) × L
    bool enableL,                     // L encoding
    bool enableMB,                    // Mixing bijections
    int Nr);
```

**Cipher Execution:**
```cpp
void Cipher(uint8_t in[16]) {
    for (int r = 0; r < Nr-1; r++) {
        ShiftRows(in);

        // Tybox lookup + XOR table chain
        for (int j = 0; j < 4; ++j) {
            aa = Tyboxes[r][j*4+0][in[j*4+0]];
            bb = Tyboxes[r][j*4+1][in[j*4+1]];
            // ... XOR tables ile combine
            // ... MBL lookup
        }
    }

    // Son round: sadece T-box
    for (int i = 0; i < 16; i++)
        in[i] = TboxesLast[i][in[i]];
}
```

### 6.3 ph4r05/Whitebox-crypto-AES Analizi

**Ek Özellikler:**
- Karroumi şeması (dual AES)
- BGE attack implementation
- Boost serialization
- External encodings

**BGE Attack:**
```cpp
// BGEAttack.cpp - 67 KB!
// Key recovery ~2^22 işlem
class BGEAttack {
    void recoverKey(WBAES& wbaes);
    void analyzeAffineRelations();
    void solveLinearEquations();
};
```

---

## 7. Önerilen Yol Haritası

### Faz 3.1a: Temel WBC Altyapısı (1 hafta)

- [ ] Cargo.toml: `whitebox` feature flag ekle
- [ ] `src/whitebox/mod.rs` modül yapısı
- [ ] `src/whitebox/tables.rs` - Chow tablo struct'ları
- [ ] `src/whitebox/cipher.rs` - WBC encrypt/decrypt
- [ ] Build.rs: `generate_whitebox_tables()` fonksiyonu
- [ ] BUILD_SEED'den WBC key derivation
- [ ] Unit testler

### Faz 3.1b: Build-Time Tablo Üretimi (1 hafta)

- [ ] T-box generation (SubBytes + AddRoundKey)
- [ ] Ty-box generation (+ MixColumns)
- [ ] XOR table generation
- [ ] Mixing bijections (L × MB)
- [ ] `whitebox_tables.rs` code generation
- [ ] NIST test vectors ile doğrulama

### Faz 3.1c: Bytecode Encryption Entegrasyonu (3-5 gün)

- [ ] Startup: WBC ile bytecode decrypt
- [ ] SMC key'i WBC ile türet
- [ ] Memory'de key obfuscation
- [ ] Integration testler

### Faz 3.1d: Optimizasyonlar (Opsiyonel)

- [ ] `whitebox_lite` - Sadece T-box (~50KB)
- [ ] Lazy table loading
- [ ] SIMD optimizasyonları (x86_64)
- [ ] Benchmark suite

---

## 8. Kaynaklar

### Akademik
- [Chow et al. 2002 - Original Paper](https://www.cs.colorado.edu/~jrblack/class/csci7000/s03/project/oorschot-whitebox.pdf)
- [Muir Tutorial 2013](https://eprint.iacr.org/2013/104.pdf)
- [BGE Attack 2004](https://link.springer.com/chapter/10.1007/978-3-540-30564-4_16)
- [DCA Attack](https://link.springer.com/chapter/10.1007/978-3-662-53140-2_11)

### Implementasyonlar
- [balena/aes-whitebox](https://github.com/balena/aes-whitebox)
- [ph4r05/Whitebox-crypto-AES](https://github.com/ph4r05/Whitebox-crypto-AES)
- [SideChannelMarvels/Deadpool](https://github.com/SideChannelMarvels/Deadpool)
- [Quarkslab Blue Galaxy Energy](https://blog.quarkslab.com/blue-galaxy-energy-a-new-white-box-cryptanalysis-open-source-tool.html)

### Diğer
- [Wikipedia - White-box cryptography](https://en.wikipedia.org/wiki/White-box_cryptography)
- [WhibOx Contest](https://whibox.io/)

---

## 9. Sonuç ve Tavsiye

**White-box cryptography tek başına güvenli DEĞİL.** Tüm akademik şemalar kırılmış durumda.

**ANCAK**, Aegis VM'in mevcut korumalarıyla kombinasyon değerli:

1. **SMC + WBC:** Tablolar sürekli encrypt/decrypt
2. **Handler Mutation + WBC:** Lookup fonksiyonları da mutate
3. **Polymorphic + WBC:** Her build farklı tablolar

**Tavsiye:** Implement et, ama "güvenlik" yerine "zorluk artırıcı" olarak konumlandır. Saldırganın işini zorlaştırır, imkansız kılmaz.

```
ROI = (Reverse Engineering Süresi Artışı) / (Implementasyon Eforu)

Tahmin: WBC ile RE süresi ~3-5x artar
        SMC+WBC ile ~10-20x artar
```

---

*Bu döküman yaşayan bir döküman olup, implementasyon sürecinde güncellenecektir.*
