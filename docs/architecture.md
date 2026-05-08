# Architecture

Hexagonal vertical-slice. Each crate is one feature, sliced into `domain/`, `application/`, `infrastructure/`. Dependency arrows always point inward.

```
infrastructure  →  application  →  domain
                                    ↑
                            (ports defined here)
```

The split also enforces compile boundaries:

- `domain/` modules only import from `ras-errors`, `ras-types`, and the crate's own domain.
- `application/` modules import the same plus the local `domain::repository` ports.
- `infrastructure/` modules implement the ports using third-party SDKs (chromiumoxide, reqwest, keyring, ...).

See [`adr/`](adr/) for decision records.
