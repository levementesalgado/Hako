# Como usar o Collatz Analyzer

## No PC (host)

```bash
# Build
cd /root/projects/mizu-os
cargo build -p collatz-analyzer

# Help
./target/debug/collatz-analyzer

# ─── Análises ────────────────────────────────────────

# 1. Animação bit-a-bit (autômato celular)
./target/debug/collatz-analyzer --cellular 27

# 2. Distribuição de v = trailing_zeros(3n+1)
./target/debug/collatz-analyzer --carry 50000

# 3. Features para treinar ML (19 colunas)
./target/debug/collatz-analyzer --ml 10000 > /tmp/collatz_ml_10k.csv

# 4. S(n) como sinal 1D para wavelet/STFT
./target/debug/collatz-analyzer --signal 100000 > /tmp/collatz_signal_100k.csv

# 5. Dataset completo (n,steps,peak,popcount,tz,residue)
./target/debug/collatz-analyzer --csv 100000 > /tmp/collatz_100k.csv

# 6. Recordistas de stopping time
./target/debug/collatz-analyzer --record 10000000

# 7. Correlação com métricas binárias
./target/debug/collatz-analyzer --correlate 50000

# 8. Análise das diferenças S(n+1)-S(n)
./target/debug/collatz-analyzer --diff 5000

# 9. Modelo preditivo (lookup mod 16)
./target/debug/collatz-analyzer --predict 50000

# 10. Fourier da paridade
./target/debug/collatz-analyzer --fourier 20000

# 11. Análise profunda de um número
./target/debug/collatz-analyzer --pattern 27
```

## No Mizu OS (via Hako)

O arquivo `hako/examples/collatz.hako` é um programa Hako que roda no kernel Mizu.
Para usá-lo:

```bash
# 1. Copia pro diretório de Hako do kernel
cp hako/examples/collatz.hako mizu-kernel/src/hako/

# 2. Adiciona no build.rs (se quiser que seja transpilado)
# Ou transpila manualmente:
cargo run -p hako -- mizu-kernel/src/hako/collatz.hako

# 3. Build do kernel
cargo build -p mizu-kernel

# 4. Roda no QEMU
qemu-system-i386 -kernel target/i686-mizu/release/mizu-kernel
```

O Hako Collatz mostra:
- Tabela de stopping times no VGA (n=1..100)
- CSV pela serial (n=1..1000) para coleta externa

## Alimentar o Mycelium-Net

```bash
# Gera o sinal de stopping times
cd /root/projects/mizu-os
./target/debug/collatz-analyzer --signal 100000 > /tmp/collatz_signal.csv

# Abre o Mycelium-Net e carrega via URL:
# file:///tmp/collatz_signal.csv
# Ou carrega na GUI com 📂 Teste (depois de copiar pra dados_test/)

# Features para treinar modelo preditivo:
./target/debug/collatz-analyzer --ml 10000 > /tmp/collatz_ml.csv
# Depois usa no Mycelium-Net modo Random Forest ou CNN
```

## Dataset disponível

| Arquivo | Linhas | Colunas |
|---------|--------|---------|
| `/tmp/collatz_100k.csv` | 100k | n,steps,peak,popcount,tz,residue2..16 |
| `/tmp/collatz_ml_10k.csv` | 10k | 19 features + target_steps |
| `/tmp/collatz_signal_100k.csv` | 100k | n,stopping_time,delta,popcount,tz,residue16 |
