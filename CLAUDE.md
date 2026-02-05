# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rust-analyzer-mcp is a Model Context Protocol (MCP) server that provides integration with rust-analyzer LSP, allowing AI assistants to analyze Rust code through standardized tools. The server acts as a bridge between MCP clients and rust-analyzer, translating MCP tool calls into LSP requests.

## Architecture

The codebase follows a modular architecture:

- **Main MCP Server** (`src/main.rs`): Handles MCP protocol, manages rust-analyzer subprocess, and routes tool calls to LSP methods
- **Test Support Library** (`test-support/`): Provides `MCPTestClient` for integration testing with proper process lifecycle management
- **Test Structure**:
  - `tests/integration/`: Core MCP server integration tests
  - `tests/stress/`: Concurrency and performance stress tests
  - `tests/unit/`: Protocol and component unit tests
  - `tests/property/`: Property-based fuzzing tests

Key architectural decisions:
- Uses Tokio async runtime for concurrent request handling
- Maintains persistent rust-analyzer subprocess for performance
- Implements proper LSP initialization sequence with workspace support
- Handles CI environment detection for test reliability

## Development Commands

### Building and Running

```bash
# Development build and run
cargo build
cargo run -- /path/to/workspace

# Release build (optimized with LTO)
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run -- /path/to/workspace
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_concurrent_tool_calls

# Run tests in release mode (for stress tests)
cargo test --release

# Run integration tests only
cargo test --test integration_tests

# Run with verbose output to debug failures
cargo test -- --nocapture

# Run tests with specific timeout debugging
RUST_BACKTRACE=1 cargo test --test integration_tests test_all_lsp_tools
```

### Linting and Formatting

```bash
# Format code
cargo +nightly fmt

# Run clippy linter
cargo clippy -- -D warnings

# Check without building
cargo check
```

## CI Considerations

The test suite includes CI-specific handling to ensure reliability in GitHub Actions:

- Tests detect CI environment via `std::env::var("CI")`
- In CI, concurrent tests run in smaller batches to avoid overwhelming the system
- Tool call timeouts are extended from 10s to 30s in CI environments
- The `test_rapid_fire_requests` test adds small delays between spawns in CI only

When debugging CI failures, check for:
- rust-analyzer initialization timeouts (30s timeout in CI)
- Concurrent request handling (batched in CI vs full concurrency locally)
- Success thresholds adjusted for CI reliability

## Test Project

The `test-project/` directory contains a minimal Rust project used for integration testing. It includes:
- Basic functions (`greet`, `Calculator` struct) for testing LSP features
- Positioned specifically to test definition, references, hover, and completion at known locations

## Key Implementation Details

### MCP Protocol Handling
- Implements full MCP initialize sequence with tool discovery
- Returns proper JSON-RPC responses with error handling
- Tools return results wrapped in MCP content items

### rust-analyzer Integration
- Spawns rust-analyzer as subprocess with stdio communication
- Implements proper LSP initialization with workspace capabilities
- Opens documents before LSP operations to ensure proper analysis
- Handles async LSP responses with request ID tracking

### Tool Reliability
- Symbols tool polls until rust-analyzer completes indexing
- Definition/references tools handle null responses during initialization
- Format tool requires document to be opened first
- Completion tool may return null during indexing

## Git Commit Conventions

Use gitmoji for commit messages. Refer to the official gimoji list at:
- Interactive picker: https://zeenix.github.io/gimoji/
- Raw database: https://zeenix.github.io/gimoji/emojis.json

## Testing Patterns

### Integration Tests
- Use `MCPTestClient::initialize_and_wait()` to ensure rust-analyzer is ready
- Check for both successful responses and null handling
- Test invalid inputs for error handling

### Stress Tests
- Test concurrent requests with `futures::future::join_all`
- Verify memory stability with repeated operations
- Test rapid-fire sequential requests for throughput

### CI-Specific Testing
```rust
// Pattern for CI-specific behavior
if std::env::var("CI").is_ok() {
    // CI-specific handling (batching, delays, extended timeouts)
} else {
    // Local development (full concurrency, normal timeouts)
}
```

---

## 🤖 MCP rust-analyzer 활용 가이드 (에이전트 최적화)

> **중요**: 코드 탐색 시 **MCP rust-analyzer 우선 사용**. Grep/Glob는 텍스트 검색에만 사용.

### ⚠️ **Workspace 설정 (스마트 버전)**

**먼저 `get_workspace`로 현재 상태 확인 → 필요할 때만 `set_workspace` 호출**

```rust
// ✅ 권장 패턴: 먼저 상태 확인
result = rust_analyzer_get_workspace()
// → {"workspace": "/path/to/project", "initialized": true}

// 다른 프로젝트면 set_workspace (같으면 스킵됨)
rust_analyzer_set_workspace("/path/to/project")
// → "Already initialized: /path/to/project (skipped)"  // 같으면 즉시 반환!
// → "Workspace set to: /path/to/new-project"           // 다르면 재초기화

workspace_symbol("CryptoWebSocketClient")
```

```rust
// ❌ 피해야 할 패턴: 매번 set_workspace 호출
rust_analyzer_set_workspace("/path/to/project")  // 매번 호출하면...
rust_analyzer_set_workspace("/path/to/project")  // → 이제 스킵됨! (개선됨)
```

**주의사항:**
- `set_workspace`는 같은 경로면 자동 스킵 (재파싱 없음)
- 새 프로젝트로 변경 시에만 파싱 시간 필요 (수초~수십초)
- `get_workspace`로 현재 상태 확인 가능: `{"workspace": "...", "initialized": true/false}`

---

### 📊 사용 가능한 도구 (사용 빈도 순)

**0. get_workspace** - 현재 상태 확인 (먼저 호출!)
**1. set_workspace** - workspace 설정 (다른 프로젝트일 때만)
2. **workspace_symbol** - 전체 심볼 검색 (파일 위치 모를 때)
2. **definition** - 정의 찾기 (Go to definition)
3. **references** - 사용처 찾기 (수정 영향 분석)
4. **hover** - 타입 정보 + 문서
5. **incoming_calls** - 누가 호출? (호출 역추적)
6. **outgoing_calls** - 뭘 호출? (의존성 파악)
7. **diagnostics** - 파일 에러/경고
8. **implementation** - Trait 구현체 찾기
9. **parent_module** - 부모 모듈 찾기
10. **inlay_hint** - 타입 힌트
11. **workspace_diagnostics** - 전체 프로젝트 진단

### 기본 워크플로우

```
0. rust_analyzer_get_workspace() → 현재 상태 확인
1. rust_analyzer_set_workspace("/path") → 필요시만 (자동 스킵됨)
2. workspace_symbol("함수명") → 위치 찾기
3. Read(파일) → 코드 읽기
4. hover → 외부 타입 확인 (Arc, DataHub 등)
5. definition → 외부 정의로 이동
6. references → 사용처 파악
7. incoming/outgoing_calls → 호출 관계 추적
8. diagnostics → 에러 확인

Note: 같은 파일 내 struct는 Read만으로 충분, hover 불필요
```

### MCP vs Grep 선택

- **코드 구조 이해**: MCP (함수, 타입, 호출 관계) ← **항상 우선!**
- **텍스트 검색**: Grep (문자열 리터럴, 주석만)

### 토큰 효율성

**모든 응답 간소화됨**:
- 85-94% 토큰 절감
- 절대경로 → 상대경로
- 필수 정보만 반환

**일일 토큰 절감**: ~500,000 토큰
