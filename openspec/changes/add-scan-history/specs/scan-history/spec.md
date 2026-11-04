# Spec Delta

## Purpose

Menyimpan hasil analisis lokal secara konsisten agar pengguna dapat meninjau ulang cakupan dan hasil lintas run tanpa mengirim data proyek keluar dari mesin.

## ADDED Requirements

### Requirement: Hasil analisis menggunakan schema berversi
Sistem MUST menyajikan run, cakupan, temuan, dan status evaluasi dalam schema berversi yang dapat dibaca consumer secara eksplisit.

#### Scenario: Consumer membaca hasil yang didukung
- **WHEN** CLI atau consumer lain membaca hasil dengan versi schema yang didukung
- **THEN** sistem menyediakan ID run, waktu, cakupan, versi engine/rule, temuan, dan status evaluasi dengan nama field yang terdokumentasi

#### Scenario: Consumer menemukan versi yang tidak dikenal
- **WHEN** consumer membaca hasil dengan versi schema yang tidak didukung
- **THEN** consumer menolak interpretasi sebagai hasil valid dan menyatakan versi yang tidak didukung

### Requirement: Hasil menyatakan cakupan serta ketidakpastian
Setiap run MUST membedakan rule yang dievaluasi, tidak berlaku, unknown, dan error serta mencantumkan file/cakupan yang dikecualikan.

#### Scenario: Bahasa atau file tidak didukung
- **WHEN** run menemukan bahasa, file, atau rule yang tidak didukung/tidak dievaluasi
- **THEN** hasil menyatakan status unknown/tidak dievaluasi dan tidak menyimpulkan proyek bersih dari cakupan tersebut

### Requirement: Riwayat disimpan hanya pada penyimpanan lokal produk
Riwayat MUST disimpan dalam database lokal produk yang terpisah dari database proyek target; penyimpanan MUST tidak memerlukan network.

#### Scenario: Simpan hasil tanpa koneksi network
- **WHEN** pengguna menyimpan run pada mode offline
- **THEN** riwayat tersimpan lokal tanpa koneksi network dan tanpa membuka database proyek target

### Requirement: Data sensitif tidak disimpan plaintext
Sebelum hasil atau evidence disimpan, sistem MUST menghapus atau meredaksi credential dan nilai secret yang terdeteksi; SQLite tidak boleh menyimpan credential konektor/provider sebagai plaintext.

#### Scenario: Temuan berisi nilai secret
- **WHEN** scanner menemukan literal credential dalam source
- **THEN** hasil yang disimpan memuat lokasi dan metadata redacted tanpa nilai credential literal

### Requirement: Migrasi dan penghapusan riwayat aman
Migrasi schema MUST transaksional, dan pengguna MUST dapat mengekspor atau menghapus riwayat lokal yang dipilih tanpa mengubah source project.

#### Scenario: Migrasi gagal
- **WHEN** migrasi schema gagal sebelum commit
- **THEN** transaksi rollback dan data riwayat sebelumnya tetap dapat dibaca

#### Scenario: Pengguna menghapus project history
- **WHEN** pengguna menghapus riwayat project lokal yang dipilih
- **THEN** hanya row yang terkait project/run terpilih dihapus dan source project tetap tidak berubah
