# Gust Cache 🦀

![gust-mascot](https://png.pngtree.com/png-vector/20240805/ourmid/pngtree-sticker-of-a-cartoon-gust-of-wind-png-image_13078480.png)

## What

**GustCache** is a lightweight, async-safe, in-memory (L1) cache for Rust.  
It supports TTL (time-to-live) and is perfect for fast, ephemeral caching in web APIs or other Rust apps.

---

## How

### 1. Add to your project

```toml
gustcache = { git = "https://github.com/Amund-Fremming/GustCache.git" }
```

### 2. Create a cache

```rust
use gustcache::GustCache;

let spin_cache = GustCache::from_ttl(chrono::Duration::minutes(2));
// or
let spin_cache = GustCache::new();
```

### 3. Use it

Pass in a key (hashed internally) and a fallback function for cache misses.  
If a valid cached value exists, it’s returned instantly. Otherwise, the fallback runs (e.g., hitting your database).

```rust
let spinners = state
    .get_spin_cache()
    .get(&request, || get_spin_page(state.get_pool(), &request))
    .await?;
```
