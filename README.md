# 🦀 Rusty-Chain

**Rusty-Chain**은 30일 완성 목표로 개발 중인 Rust 기반의 경량 블록체인 프로젝트입니다. 단순한 학습용 코드를 넘어, 실제 네트워크 동기화와 트랜잭션 전파가 가능한 데모 시스템 구성을 목표로 합니다.

## 🚀 프로젝트 개요
이 프로젝트는 블록체인의 핵심 원리를 Rust 언어로 직접 구현하며 학습하는 것을 목적으로 합니다. 단일 노드 MVP에서 시작하여, 현재는 P2P 네트워크 전파 로직까지 구현된 상태입니다.

- **목표:** 3노드 이상의 Docker 환경에서 실시간 트랜잭션 전파 및 체인 동기화 데모.
- **철학:** 코드 가독성 우선, 명확한 책임 분리(Core vs Network), 철저한 테스트 기반 개발.

## 🛠 주요 기능
- **블록체인 코어:** 블록 구조, SHA-256 해시, Merkle Root 계산.
- **합의 알고리즘:** Proof of Work (PoW) 기반 채굴 및 난이도 조절.
- **계정 모델:** Ed25519 타원 곡선 암호화 기반 키 쌍 생성 및 트랜잭션 서명/검증.
- **상태 관리:** 이중 지불 방지를 위한 넌스(Nonce) 관리 및 잔액(Balance) 검증.
- **P2P 네트워크:** Gossip 프로토콜 기반 트랜잭션/블록 전파 및 노드 주소 교환.

## 📂 프로젝트 구조
코드는 책임별로 모듈화되어 있습니다:
```text
src/
├── core/               # 블록체인 핵심 로직
│   ├── chain.rs        # 체인 저장/로드 및 관리
│   ├── mempool.rs      # 미확정 트랜잭션 풀
│   ├── state.rs        # 계정 잔액 및 넌스 상태 추적
│   ├── keys.rs         # Ed25519 지갑 관리
│   ├── crypto.rs       # 서명 및 검증 유틸리티
│   ├── network.rs      # P2P 메시지 정의
│   ├── p2p.rs          # 노드 핸들링 및 가십 프로토콜
│   └── types.rs        # 공통 데이터 구조체
├── lib.rs              # 라이브러리 엔트리포인트
└── main.rs             # CLI 인터페이스 구현
```

## ⚙️ 설치 및 실행

### 필수 요구 사항
- **Rust Toolchain:** [rustup](https://rustup.rs/)을 통해 설치 (Edition 2021)

### 클론 및 빌드
```bash
git clone https://github.com/surfsurfmasurf/rusty-chain.git
cd rusty-chain
cargo build
```

### 테스트 실행
```bash
# 단위 테스트 및 통합 테스트 실행
cargo test
```

## 💻 사용 가이드 (CLI)

### 1. 지갑 생성
```bash
# 'alice'라는 이름의 키 페어 생성 (data/keys/alice.json)
cargo run -- keygen --name alice

# 공개 키(주소) 확인
cargo run -- addr --name alice
```

### 2. 체인 초기화 및 상태 확인
```bash
# 제네시스 블록 생성
cargo run -- init

# 현재 체인 상태(높이, 난이도, 트랜잭션 수) 확인
cargo run -- status
```

### 3. 트랜잭션 생성 및 서명
```bash
# 서명된 트랜잭션을 멤풀에 추가
cargo run -- tx-add --signer alice --to <보낼_주소> --amount 10
```

### 4. 블록 채굴 (PoW)
```bash
# 멤풀의 트랜잭션을 포함하여 새 블록 채굴
cargo run -- mine --difficulty 3 --miner alice
```

### 5. 체인 검증
```bash
# 전체 체인의 무결성(해시 연결, PoW, 서명, 넌스) 검증
cargo run -- validate
```

## 📅 로드맵
- [x] **Week 1:** 단일 노드 MVP (블록, 해시, PoW)
- [x] **Week 2:** 계정 모델 및 암호화 서명 도입
- [x] **Week 3:** P2P 가십 프로토콜 및 실시간 전파
- [x] **Week 4:** 최종 안정화 및 3-노드 데모 시스템 완성
- [x] **RBF (Replace-By-Fee):** 트랜잭션 시퀀스 및 수수료 기반 대체 로직 구현
- [x] **P2P Rejection:** 잘못된 RBF 요청에 대한 네트워크 거부 메시지 처리
- [x] **CLI Enhancement:** `tx-add`에서 시퀀스 번호 지정 지원
- [x] **Mempool TTL:** 트랜잭션 타임스탬프 기반 만료 및 `tx-evict` 명령 추가

---
*본 프로젝트는 30일 챌린지 기간 내에 계획된 모든 핵심 기능 개발을 완료하였습니다.*

## 📜 라이선스
MIT License.
