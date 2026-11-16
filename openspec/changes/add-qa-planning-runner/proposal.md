# Proposal

## Why

SNIPE-CODE belum menghasilkan test case/script terstruktur dan tidak mengisolasi command test yang dapat mengubah file, mengakses network, atau memakai DB target proyek. Pisahkan perencanaan read-only dari eksekusi berizin agar test yang dihasilkan bisa ditinjau dan efek run terukur.

## What Changes

- Rencanakan positive/negative, boundary, auth/permission, validation/error, dan regression case dengan trace requirement-to-case.
- Generate script hanya untuk adapter yang dinyatakan didukung; validasi struktur sebelum menyebut script siap, tanpa menganggap test lulus.
- Jadikan eksekusi tahap eksplisit terpisah dengan review command, path, endpoint, environment, mount, network, dan output.
- Blokir sebelum spawn bila konfigurasi DB eksplisit remote, production-like, atau ambigu. Jika konfigurasi DB tidak terdeteksi, tampilkan peringatan dan minta approval CLI eksplisit; jangan klaim DB tidak ada atau aman. Bersihkan environment DB host/target yang dikenali. Eksekusi bukan sandbox kecuali boundary OS benar-benar diterapkan; ungkapkan risiko filesystem, network, endpoint, dan DB hardcoded.
- Rekam exit, timeout, log tersanitasi, evidence, skipped/unavailable/blocked, serta batas cleanup.
- Sediakan CLI tanpa GUI dan agent dengan skills, termasuk testing skill untuk proyek pengguna; uji kontrak kemampuan agent dan CLI; UAT manusia per OS dilaporkan terpisah bila tersedia.
- Dukung AI dan MCP lokal maupun online secara opsional, tanpa menjadi prasyarat; preview dan persetujuan eksplisit diperlukan sebelum data dikirim, endpoint diakses, tool di-install, atau test dijalankan.
- Rencanakan dukungan bertahap untuk unit/regression, Playwright, Postman melalui Postman CLI, Bruno, dan Maestro; jangan klaim adapter tersedia sebelum diimplementasikan dan diverifikasi.

## Capabilities

### New Capabilities
- `qa-planning`: Perencanaan test case/script yang dapat ditinjau dan tidak mengeksekusi target.
- `qa-runner`: Eksekusi test setelah approval scope-bound dengan mode host atau sandbox yang dibedakan jujur; preflight data tetap fail closed pada DB eksplisit yang berisiko. Host execution bukan sandbox.

### Modified Capabilities

## CR alignment

See `../../CR-ALIGNMENT.md`. Postman CLI is the first adapter pilot; the unit/regression task defines its subsequent framework slice, not an earlier competing pilot. Setup preview and approval are separate from run approval. Full M5 provider and service-specific connector runtime is deferred pending explicit scope and security contracts; optional planning integration requirements do not claim those services exist.

## Impact

Menyentuh CLI/agent, skills, runner, model hasil, policy izin, adapters test, serta unit/contract/security tests. Rencana case dapat memakai schema hasil dari `add-scan-history`; runner tidak bergantung pada GUI, AI, atau connector. Host mode yang mendapat approval dapat berjalan dengan disclosure non-isolated; hanya mode yang secara eksplisit meminta sandbox tanpa OS boundary memadai yang `blocked`, tanpa downgrade diam-diam ke host. Scope adapter dan perilaku eksekusi DB harus ditetapkan sebelum implementation.
