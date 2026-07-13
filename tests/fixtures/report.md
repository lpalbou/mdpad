# Ollama benchmark — vps-3

Generated 2026-07-13T03:50:14+00:00 — regenerate anytime with `python3 scripts/make_report.py --host-label vps-3`.

## Environment
- CPU: Intel Core Processor (Haswell, no TSX) × 6 vCPU
- MemTotal:       11951484 kB | available at start: 7.31 GiB
- Kernel 7.0.0-27-generic | ollama 0.24.0
- Noise: GitLab (puma, sidekiq, gitaly) and agent CLIs run on this host during the benchmark; they hold ~3-4 GiB RAM and burn CPU intermittently. Numbers include that real-world noise.

## Method
- Prompts frozen in `prompts/` (sha256 in `prompts_meta.json`); temperature 0, seed 42, num_predict 400.
- Metrics from ollama's server counters; TTFT and wall time measured client-side on streamed responses; tests run on a warm model (cold load measured separately).
- prefill tok/s = prompt processing speed; gen tok/s = generation speed.

## Test a_short — nominal 100 prompt tokens (num_ctx 4096, num_predict 400)

| Model | Size | Status | Prompt tok | Prefill tok/s | Gen tok | Gen tok/s | TTFT s | Wall s | Done |
|---|---|---|---|---|---|---|---|---|---|
| LiquidAI/LFM2.5-350M:latest | 379 MB | ok | 97 | 215.08 | 73 | 40.58 | 0.585 | 2.417 | stop |
| openbmb/minicpm5:latest | 688 MB | ok | 78 | 97.87 | 400 | 21.66 | 1.021 | 20.505 | length |
| LiquidAI/lfm2.5-1.2b-instruct:latest | 730 MB | ok | 97 | 72.28 | 170 | 18.6 | 1.528 | 10.725 | stop |
| lfm2.5-thinking:1.2b | 731 MB | ok | 82 | 59.64 | 400 | 15.19 | 1.614 | 28.091 | length |
| gemma3:1b | 815 MB | ok | 85 | 65.33 | 228 | 14.12 | 1.897 | 18.349 | stop |
| qwen3.5:0.8b | 1.0 GB | ok | 87 | 76.32 | 400 | 8.03 | 1.773 | 52.091 | length |
| qwen3.5:2b-q4_K_M | 1.9 GB | ok | 87 | 46.57 | 400 | 8.68 | 2.414 | 49.02 | length |
| granite4.1:3b | 2.1 GB | ok | 79 | 20.98 | 154 | 11.36 | 3.898 | 17.739 | stop |
| qwen3.5:2b | 2.7 GB | ok | 87 | 47.13 | 400 | 7.52 | 2.397 | 56.071 | length |
| qwen3.5:4b | 3.4 GB | ok | 87 | 18.74 | 400 | 3.58 | 5.249 | 117.279 | length |
| maternion/lfm2.5:latest | 5.2 GB | ok | 79 | 51.87 | 400 | 11.45 | 1.95 | 37.099 | length |
| granite4.1:8b | 5.3 GB | ok | 79 | 9.24 | 400 | 3.23 | 8.703 | 133.237 | length |
| ministral-3:8b | 6.0 GB | ok | 626 | 79.66 | 239 | 4.39 | 8.363 | 62.922 | stop |
| qwen3.5:9b | 6.6 GB | ok | 87 | 9.3 | 400 | 2.68 | 10.001 | 159.952 | length |
| gemma4:e2b | 7.2 GB | ok | 92 | 45.25 | 400 | 10.41 | — | 41.512 | length |
| nomic-embed-text:latest | 274 MB | ok | 67 | — | — | — | — | 0.206 | — |
| minimax-m3:cloud | cloud | skipped_model_broken | — | — | — | — | — | 0.15 | — |

## Test b_medium — nominal 1000 prompt tokens (num_ctx 4096, num_predict 400)

| Model | Size | Status | Prompt tok | Prefill tok/s | Gen tok | Gen tok/s | TTFT s | Wall s | Done |
|---|---|---|---|---|---|---|---|---|---|
| LiquidAI/LFM2.5-350M:latest | 379 MB | ok | 1017 | 240.14 | 130 | 16.91 | 4.353 | 12.103 | stop |
| openbmb/minicpm5:latest | 688 MB | ok | 944 | 87.7 | 400 | 19.17 | 10.962 | 32.68 | length |
| LiquidAI/lfm2.5-1.2b-instruct:latest | 730 MB | ok | 1017 | 70.16 | 251 | 17.66 | 14.613 | 28.933 | stop |
| lfm2.5-thinking:1.2b | 731 MB | ok | 1002 | 71.29 | 400 | 15.55 | 14.203 | 40.093 | length |
| gemma3:1b | 815 MB | ok | 1046 | 74.51 | 400 | 14.97 | 14.577 | 41.834 | length |
| qwen3.5:0.8b | 1.0 GB | ok | 1041 | 77.45 | 400 | 7.35 | 13.982 | 68.914 | length |
| qwen3.5:2b-q4_K_M | 1.9 GB | ok | 1041 | 46.03 | 400 | 8.44 | 23.122 | 71.022 | length |
| granite4.1:3b | 2.1 GB | ok | 915 | 21.27 | 400 | 7.89 | 43.221 | 94.603 | length |
| qwen3.5:2b | 2.7 GB | ok | 1041 | 45.39 | 400 | 7.1 | 23.446 | 80.282 | length |
| qwen3.5:4b | 3.4 GB | ok | 1041 | 17.66 | 400 | 3.79 | 59.404 | 165.227 | length |
| maternion/lfm2.5:latest | 5.2 GB | ok | 962 | 47.64 | 400 | 12.37 | 20.551 | 53.116 | length |
| granite4.1:8b | 5.3 GB | ok | 915 | 8.89 | 400 | 3.06 | 103.069 | 234.411 | length |
| ministral-3:8b | 6.0 GB | ok | 1589 | 14.19 | 400 | 3.61 | 112.511 | 223.696 | length |
| qwen3.5:9b | 6.6 GB | ok | 1041 | 10.32 | 400 | 2.63 | 101.495 | 254.1 | length |
| gemma4:e2b | 7.2 GB | ok | 1053 | 37.07 | 400 | 9.25 | — | 72.884 | length |
| nomic-embed-text:latest | 274 MB | ok | 842 | — | — | — | — | 1.979 | — |
| minimax-m3:cloud | cloud | skipped_model_broken | — | — | — | — | — | 0.15 | — |

## Test c_long — nominal 10000 prompt tokens (num_ctx 13312, num_predict 400)

| Model | Size | Status | Prompt tok | Prefill tok/s | Gen tok | Gen tok/s | TTFT s | Wall s | Done |
|---|---|---|---|---|---|---|---|---|---|
| LiquidAI/LFM2.5-350M:latest | 379 MB | ok | 9143 | 92.32 | 165 | 12.75 | 100.002 | 113.062 | stop |
| openbmb/minicpm5:latest | 688 MB | ok | 8702 | 32.13 | 400 | 8.68 | 272.847 | 319.8 | length |
| LiquidAI/lfm2.5-1.2b-instruct:latest | 730 MB | ok | 9143 | 37.05 | 313 | 8.83 | 248.27 | 283.907 | stop |
| lfm2.5-thinking:1.2b | 731 MB | ok | 9128 | 37.22 | 400 | 9.03 | 246.527 | 290.996 | length |
| gemma3:1b | 815 MB | ok | 10121 | 65.55 | 400 | 12.62 | 156.911 | 189.197 | length |
| qwen3.5:0.8b | 1.0 GB | ok | 9937 | 61.61 | 400 | 7.82 | 166.646 | 218.363 | length |
| qwen3.5:2b-q4_K_M | 1.9 GB | ok | 9937 | 38.17 | 400 | 6.8 | 266.404 | 325.792 | length |
| granite4.1:3b | 2.1 GB | ok | 8699 | 7.61 | 400 | 2.25 | 1148.099 | 1326.41 | length |
| qwen3.5:2b | 2.7 GB | ok | 9937 | 39.56 | 400 | 5.95 | 257.275 | 325.082 | length |
| qwen3.5:4b | 3.4 GB | ok | 9937 | 11.24 | 400 | 2.03 | 891.186 | 1088.517 | length |
| maternion/lfm2.5:latest | 5.2 GB | ok | 8795 | 29.99 | 400 | 6.34 | 297.548 | 360.931 | length |
| granite4.1:8b | 5.3 GB | ok | 8699 | 4689.49 | 400 | 1.11 | 723.214 | 1084.288 | length |
| ministral-3:8b | 6.0 GB | error | — | — | — | — | — | 1.806 | — |
| qwen3.5:9b | 6.6 GB | error | — | — | — | — | — | 2.29 | — |
| gemma4:e2b | 7.2 GB | ok | 10128 | 27.49 | 400 | 7.72 | — | 431.149 | length |
| nomic-embed-text:latest | 274 MB | error | — | — | — | — | — | 0.034 | — |
| minimax-m3:cloud | cloud | skipped_model_broken | — | — | — | — | — | 0.15 | — |

## Cold model load (first request after unload)

| Model | Load s | Warmup status |
|---|---|---|
| LiquidAI/LFM2.5-350M:latest | 0.745 | ok |
| openbmb/minicpm5:latest | 2.455 | ok |
| LiquidAI/lfm2.5-1.2b-instruct:latest | 1.123 | ok |
| lfm2.5-thinking:1.2b | 1.361 | ok |
| gemma3:1b | 2.769 | ok |
| qwen3.5:0.8b | 4.78 | ok |
| qwen3.5:2b-q4_K_M | 4.778 | ok |
| granite4.1:3b | 4.665 | ok |
| qwen3.5:2b | 5.489 | ok |
| qwen3.5:4b | 6.366 | ok |
| maternion/lfm2.5:latest | 3.835 | ok |
| granite4.1:8b | 7.409 | ok |
| ministral-3:8b | 7.254 | ok |
| qwen3.5:9b | 8.152 | ok |
| gemma4:e2b | 9.544 | ok |
| nomic-embed-text:latest | 0.5 | ok |
| minimax-m3:cloud | -0.0 | error |

## Models/requests that did not work

| Model | Test | Attempt | Status | Error |
|---|---|---|---|---|
| granite4.1:8b | c_long | 1 | timeout | TimeoutError: timed out |
| ministral-3:8b | c_long | 1 | error | HTTP 500: {"error":"model requires more system memory (8.1 GiB) than is available (8.0 GiB)"} |
| ministral-3:8b | c_long | 2 | error | HTTP 500: {"error":"model requires more system memory (8.1 GiB) than is available (8.0 GiB)"} |
| qwen3.5:9b | c_long | 1 | error | HTTP 500: {"error":"model requires more system memory (8.3 GiB) than is available (8.0 GiB)"} |
| qwen3.5:9b | c_long | 2 | error | HTTP 500: {"error":"model requires more system memory (8.3 GiB) than is available (8.1 GiB)"} |
| nomic-embed-text:latest | c_long | 1 | error | HTTP 400: {"error":"the input length exceeds the context length"} |
| minimax-m3:cloud | a_short | 0 | skipped_model_broken | HTTP 401: {"error":"Unauthorized"}  |
| minimax-m3:cloud | b_medium | 0 | skipped_model_broken | HTTP 401: {"error":"Unauthorized"}  |
| minimax-m3:cloud | c_long | 0 | skipped_model_broken | HTTP 401: {"error":"Unauthorized"}  |

## Notes
- openbmb/minicpm5:latest: same digest as openbmb/minicpm5:q4_K_M (08239e8f70e0), benchmarked once
- gemma4:e2b: thinking model: on ollama 0.24.0 /api/generate returns NO text (server counters still valid); /api/chat exposes the text in the 'thinking' field. Verbatims are therefore empty — expected.
- granite4.1:8b c_long: succeeded on attempt 2 — prefill 4689.49 tok/s is cache-assisted after the failed attempt; only its generation speed is trustworthy.
- ministral-3:8b shows inflated prompt token counts (~+550): its chat template injects a large default system prompt; counts are recorded as measured.
