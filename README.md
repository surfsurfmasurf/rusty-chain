# rusty-chain

30일 동안 Rust로 **데모 가능한 미니 블록체인**을 만드는 프로젝트.

## 목표(30일)
- Rust로 작성된 단일 레포
- 3노드 데모(최종 목표: `docker compose up`)
- 트랜잭션 전송 → mempool → 블록 생성 → 전파 → 체인 동기화
- 문서/테스트/릴리즈까지 포함

## 개발 원칙
- 매일 최소 2~3 커밋 (feat/refactor/docs/test로 쪼개기)
- `PROGRESS.md`에 일일 진행 상황 기록
- 단순하고 명확한 규칙을 우선(학습/데모 목적)

## 빠른 시작
```bash
cargo run -- --help
```

## 로드맵(요약)
- Week 1: 단일 노드 체인 MVP(블록/해시/검증/PoW/CLI)
- Week 2: 트랜잭션 + 서명 + 지갑(계정 모델)
- Week 3: P2P 전파/동기화 + 포크 처리
- Week 4: 안정화 + docker 데모 + 문서/릴리즈


## CLI

```bash
# create genesis
cargo run -- init

# show status
cargo run -- status

# validate chain invariants
cargo run -- validate

# mine 1 empty block (demo PoW)
cargo run -- mine --difficulty 3
```


### Demo
```bash
# write genesis (default: data/chain.json)
cargo run -- init

# read status
cargo run -- status

# validate
cargo run -- validate

# mine + check status
cargo run -- mine --difficulty 2
cargo run -- status

# or explicit path
cargo run -- init --path /tmp/chain.json
cargo run -- mine --difficulty 2 --path /tmp/chain.json
cargo run -- status --path /tmp/chain.json
```
