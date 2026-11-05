# Proposal

## Why

SNIPE-CODE belum menghasilkan test case/script terstruktur dan tidak mengisolasi command test yang dapat mengubah file, mengakses network, atau memakai DB target proyek. Pisahkan perencanaan read-only dari eksekusi berizin agar test yang dihasilkan bisa ditinjau dan efek run terukur.

## What Changes

- Rencanakan positive/negative, boundary, auth/permission, validation/error, dan regression case dengan trace requirement-to-case.
- Generate script hanya untuk adapter yang dinyatakan didukung; validasi struktur sebelum menyebut script siap, tanpa menganggap test lulus.
- Jadikan eksekusi tahap eksplisit terpisah dengan review command, path, endpoint, environment, mount, network, dan output.
- Blokir DB produksi/remote/ambigu dan eksekusi bila isolasi filesystem/proses/network tidak dapat dibuktikan; tidak menginstal tool diam-diam.
- Rekam exit, timeout, log tersanitasi, evidence, skipped/unavailable/blocked, serta batas cleanup.
- Sediakan CLI tanpa GUI dan agent dengan skills, termasuk testing skill untuk proyek pengguna; kemampuan agent dan CLI harus setara dan diuji melalui UAT per OS.
- Dukung AI dan MCP lokal maupun online secara opsional, tanpa menjadi prasyarat; preview dan persetujuan eksplisit diperlukan sebelum data dikirim, endpoint diakses, tool di-install, atau test dijalankan.
- Rencanakan dukungan bertahap untuk unit/regression, Playwright, Postman melalui Postman CLI, Bruno, dan Maestro; jangan klaim adapter tersedia sebelum diimplementasikan dan diverifikasi.

## Capabilities

### New Capabilities
- `qa-planning`: Perencanaan test case/script yang dapat ditinjau dan tidak mengeksekusi target.
- `qa-runner`: Eksekusi test yang disetujui dalam batas isolasi/DB yang dinyatakan dan fail closed.

### Modified Capabilities

## Impact

Menyentuh CLI/agent, skills, runner, model hasil, policy izin, adapters test, serta unit/contract/security tests. Rencana case dapat memakai schema hasil dari `add-scan-history`; runner tidak bergantung pada GUI, AI, atau connector. Platform yang tak memiliki sandbox memadai harus mengembalikan `blocked`, bukan melonggarkan isolasi. Scope adapter dan perilaku eksekusi DB harus ditetapkan sebelum implementation.
