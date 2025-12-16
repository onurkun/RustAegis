# ⚡ Async VM Roadmap (Experimental Feature)

> **Hedef:** Aegis VM motorunu opsiyonel olarak `async/await` mimarisine geçirerek, kod akışını (Control Flow) zorlaştırmak ve zamanlama tabanlı anti-analiz tekniklerini güçlendirmek.

---

## 🎯 Threat Model & Hedefler

Normal (Senkron) bir VM döngüsü şöyledir:
`Fetch -> Decode -> Execute -> Loop`

Bu döngü, debugger ile "Step Over" yapıldığında tahmin edilebilir bir sırayla ilerler.

**Async VM** ise şöyledir:
`Fetch -> Decode -> Await (Yield) -> Executor (State Machine Transition) -> ... -> Execute`

### Neyi Zorlaştırır? (Hardening)
1.  **Analiz Karmaşası:** Rust compiler'ı `async fn`'i devasa bir `enum` (State Machine) haline getirir. Debugger ile takip eden kişi, sürekli Executor koduna düşer ve asıl mantığı kaybeder.
2.  **Zamanlama Manipülasyonu:** Araya rastgele `yield` (bekleme) atarak zamanlama analizlerini (Timing Attack) bozabiliriz.
3.  **State Machine Obfuscation:** Rust compiler'ı `async fn`'i devasa bir `enum` (State Machine) haline getirir. Bu, tersine mühendislik araçlarının (IDA/Ghidra) akış grafiğini (CFG) çizmesini zorlaştırır.

---

## 🛠️ Teknik Mimari

### 1. Feature Flag Sistemi
`Cargo.toml`:
```toml
[features]
default = ["std", "whitebox"]
# Deneysel: Async VM motorunu aktif eder (Custom Executor ile)
async_vm = [] 
```

### 2. Executor Seçimi: "Micro Custom Executor"
Mobil cihazlarda pil tüketimini ve ısınmayı önlemek için "Busy Spin" yapmayan, sadece State Machine'i ilerleten minimal bir yapı.

**Tasarım Prensipleri:**
*   **No Busy Spin (Düzeltildi):** `Poll::Pending` durumunda `std` varsa `std::thread::yield_now()` kullanır. `no_std` için `core::hint::spin_loop()` kullanılır (bir hint, %100 CPU kullanımı değil, sadece CPU'ya "şu an işim yok ama ilerliyorum" der).
*   **Waker-less:** Tek thread çalıştığı için karmaşık `Waker` mantığına gerek yoktur. `block_on` döngüsü state'i ilerletir.

```rust
// aegis_vm/src/async_utils.rs

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// noop_waker manuel implementasyonu (stable Rust için)
const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RAW_WAKER_INSTANCE, // clone
    |_| {},                 // wake
    |_| {},                 // wake_by_ref
    |_| {},                 // drop
);
const RAW_WAKER_INSTANCE: RawWaker = RawWaker::new(core::ptr::null(), &RAW_WAKER_VTABLE);

fn noop_waker() -> Waker {
    unsafe { Waker::from_raw(RAW_WAKER_INSTANCE) }
}

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        // Future'ı pinle (Async safety - Stack Pinning)
        // INVARIANT: future stack üzerinde hareket etmemeli.
        let pinned = unsafe { Pin::new_unchecked(&mut future) };
        
        match pinned.poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {
                // Anti-Analiz: Bekleme sırasında CPU'ya hint ver veya yield yap
                #[cfg(feature = "std")]
                std::thread::yield_now();
                
                // no_std ortamında aktif bekleme yerine CPU'ya "işlem yapabilirsin" hinti ver
                // Bu, %100 CPU kullanımını azaltır, pil dostudur.
                #[cfg(not(feature = "std"))]
                core::hint::spin_loop(); 
            }
        }
    }
}
```

### 3. Engine Değişimi & Yield Stratejisi

**Kritik:** Her opcode'da yield yapmak performansı öldürür.

```rust
// Asenkron Versiyon (Paranoid)
#[cfg(feature = "async_vm")]
pub async fn run(state: &mut VmState) -> VmResult<()> {
    // build_config'den türetilen rastgele maske (Polimorfizm)
    // Örn: 0xFF (256 adımda bir), 0x7F (128 adımda bir)
    // state.build_config.yield_mask (VmState'e eklenecek)
    let yield_mask = state.get_yield_mask(); // VmState metoduna taşındı

    while !state.halted {
        dispatch(state)?;
        
        // Anti-Analiz: Kontrollü Yield
        // Her N adımda bir yield yaparak state machine'i kırar.
        // Bu sayede performans kaybı %1-5 seviyesinde tutulur.
        if (state.instruction_count & yield_mask) == 0 {
            YieldNow { yielded: false }.await; // Default derive yerine explicit init
        }
    }
    Ok(())
}

// Basit Yield Future (Wake çağırmaz, sadece Pending döner)
#[derive(Default)] // Default derive eklendi
struct YieldNow { yielded: bool }
impl Future for YieldNow {
    type Output = ();
    // _cx parametresi kullanılmıyor
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            // Waker çağırmaya gerek yok, block_on döngüsü zaten dönecek
            Poll::Pending 
        }
    }
}
```

---

## 📅 Uygulama Planı

### Phase 1: Custom Executor
*   `src/async_utils.rs` oluşturulacak.
*   `no_std` uyumlu, pil dostu `block_on` implemente edilecek.

### Phase 2: Async Engine Implementasyonu
*   `src/engine.rs` dosyasına `async fn run()` eklenecek.
*   `execute` fonksiyonu feature flag ile güncellenecek.

### Phase 3: Yield Stratejisi
*   `build.rs` içinde `YIELD_MASK` sabiti üretilecek (Polimorfizm).
*   `VmState` yapısına `yield_mask: u8` alanı ve `get_yield_mask()` metodu eklenecek.
*   Engine içinde bu maske kullanılarak seyrek yield yapılacak.

### Phase 4: Test & Benchmark
*   **Determinism Test:** Async ve Sync versiyonlar aynı bytecode için aynı sonucu veriyor mu?
*   **Overhead Test:** `%X` performans kaybı ve pil etkisi ölçülecek.

---

## ⚠️ Güvenlik ve Safety Notları

1.  **Safety:** `unsafe { Pin::new_unchecked }` kullanımı, future'ın stack'te hareket etmeyeceği varsayımına dayanır. `block_on` yapımızda bu güvenlidir.
2.  **Pil Ömrü:** Yanlış `spin_loop` kullanımı mobil cihazlarda pil tüketimini artırır. `YieldNow` sadece state geçişi için kullanılmalı, bekleme için değil. `core::hint::spin_loop()` CPU'ya "hiçbir şey yapmıyorum, uyuyabilirsin" sinyali verir, tam bir busy-loop değildir.
3.  **VmState Send:** Tek thread'li executor kullandığımız için `VmState`'in `Send` olması zorunlu değildir, ancak derleyici (compiler) async bloklar için bunu isteyebilir. Gerekirse `!Send` wrapper kullanılabilir.

---

*Son Güncelleme: 2025*
*Revize: AI Review Sonrası*