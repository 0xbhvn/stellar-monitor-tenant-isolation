# Testing Guide: Achieving 95%+ Coverage

## Current State vs OpenZeppelin Monitor Standards

### What We Currently Have (13.42% Coverage)

- Basic model unit tests testing getters and simple methods
- Enum serialization/deserialization tests
- Simple struct creation tests
- Configuration validation tests

### What OpenZeppelin Monitor Has (96% Coverage)

- Comprehensive unit tests for every module
- Integration tests for complex workflows
- Property-based tests for invariants
- Mock implementations for all external dependencies
- Test builders and fixtures for reusable test data
- Performance benchmarks
- Error injection and failure recovery tests

## Key Differences and Learnings

### 1. Test Organization

**OpenZeppelin Monitor Structure:**

```bash
tests/
├── integration/
│   ├── blockchain/        # Tests for blockchain clients
│   ├── blockwatcher/      # Service integration tests
│   ├── filters/           # Filter evaluation tests
│   ├── fixtures/          # Reusable test data
│   ├── mocks/            # Mock implementations
│   ├── monitor/          # Monitor execution tests
│   ├── notifications/    # Notification service tests
│   └── security/         # Security-focused tests
├── properties/           # Property-based tests
└── integration.rs        # Main integration test entry
```

**Our Current Structure:**

```bash
tests/
├── integration/          # Basic API tests
├── unit/                # Model tests only
└── model_tests.rs       # Additional model tests
```

### 2. Testing Philosophy

**OpenZeppelin Monitor:**

- **Feature-based organization**: Tests grouped by functionality, not by code structure
- **Mock everything external**: Database, HTTP clients, file systems
- **Test data builders**: Fluent APIs for creating test objects
- **Fixtures for consistency**: Shared test data in JSON files
- **Property-based testing**: Test invariants, not just examples

**Our Current Approach:**

- Layer-based organization (unit/integration)
- Direct database connections in tests
- Inline test data creation
- Example-based testing only

### 3. What Makes Their Tests Comprehensive

#### a) Mock Infrastructure

```rust
// OpenZeppelin uses mockall for everything
mock! {
    pub BlockchainClient {}
    
    #[async_trait]
    impl BlockchainClientTrait for BlockchainClient {
        async fn get_block(&self, number: u64) -> Result<Block>;
        async fn get_transaction(&self, hash: H256) -> Result<Transaction>;
    }
}
```

#### b) Test Builders

```rust
// Fluent API for test data
let monitor = MonitorBuilder::new()
    .with_name("Test Monitor")
    .with_network_id("ethereum-mainnet")
    .with_filter(FilterBuilder::new()
        .with_event("Transfer")
        .with_address("0x123...")
        .build())
    .build();
```

#### c) Comprehensive Error Testing

```rust
#[test]
async fn test_network_timeout() {
    let mut client = MockBlockchainClient::new();
    client.expect_get_block()
        .times(3)  // Retry 3 times
        .returning(|_| Err(NetworkError::Timeout));
    
    // Test that service handles timeout gracefully
}
```

#### d) Property-Based Tests

```rust
proptest! {
    #[test]
    fn test_filter_evaluation_properties(
        block_number in 0u64..1_000_000,
        gas_price in 0u128..1_000_000_000_000
    ) {
        // Test that filter evaluation is deterministic
        let filter = create_filter();
        let result1 = filter.evaluate(block_number, gas_price);
        let result2 = filter.evaluate(block_number, gas_price);
        assert_eq!(result1, result2);
    }
}
```

## Roadmap to 95% Coverage

### Phase 1: Infrastructure Setup (Current: 13% → Target: 20%)

- [x] Model unit tests
- [ ] Test utilities module with builders
- [ ] Mock infrastructure using mockall
- [ ] Fixtures for test data

### Phase 2: Repository Layer (20% → 35%)

- [ ] Mock database connections
- [ ] Test all CRUD operations
- [ ] Test error conditions (connection failures, constraints)
- [ ] Test concurrent access and transactions

### Phase 3: Service Layer (35% → 50%)

- [ ] Mock repository dependencies
- [ ] Test business logic thoroughly
- [ ] Test error propagation
- [ ] Test resource quota enforcement

### Phase 4: API Layer (50% → 65%)

- [ ] HTTP request/response tests
- [ ] Authentication and authorization tests
- [ ] Rate limiting tests
- [ ] Input validation tests

### Phase 5: Integration Tests (65% → 80%)

- [ ] Full workflow tests (create tenant → add users → create monitors)
- [ ] Cross-tenant isolation tests
- [ ] Database migration tests
- [ ] API integration tests

### Phase 6: Edge Cases & Security (80% → 90%)

- [ ] SQL injection attempts
- [ ] Invalid JWT tokens
- [ ] Malformed requests
- [ ] Resource exhaustion
- [ ] Race conditions

### Phase 7: Final Push (90% → 95%+)

- [ ] Property-based tests for invariants
- [ ] Performance benchmarks
- [ ] Documentation tests
- [ ] Example code in docs

## Best Practices from OpenZeppelin Monitor

### 1. Never Test Against Real Services

```rust
// Bad
#[test]
async fn test_monitor_creation() {
    let pool = create_real_db_connection().await;
    // This is slow and flaky
}

// Good
#[test]
async fn test_monitor_creation() {
    let mut repo = MockMonitorRepository::new();
    repo.expect_create()
        .returning(|_| Ok(test_monitor()));
    // Fast and reliable
}
```

### 2. Use Builders for Test Data

```rust
// Bad
let tenant = Tenant {
    id: Uuid::new_v4(),
    name: "Test".to_string(),
    slug: "test".to_string(),
    // ... 10 more fields
};

// Good
let tenant = TenantBuilder::new()
    .with_name("Test")
    .build();
```

### 3. Test One Thing at a Time

```rust
// Bad
#[test]
fn test_tenant_operations() {
    // Tests creation, update, and deletion in one test
}

// Good
#[test]
fn test_tenant_creation() { /* ... */ }

#[test]
fn test_tenant_update() { /* ... */ }

#[test]
fn test_tenant_deletion() { /* ... */ }
```

### 4. Test Error Paths Explicitly

```rust
#[test]
async fn test_monitor_creation_quota_exceeded() {
    let service = create_service_with_quota(max_monitors: 0);
    let result = service.create_monitor(...).await;
    assert!(matches!(result, Err(QuotaExceeded)));
}
```

### 5. Use Fixtures for Complex Data

```json
// tests/fixtures/monitors/complex_filter.json
{
  "type": "ethereum_event",
  "contract": "0x...",
  "event": "Transfer(address,address,uint256)",
  "filters": {
    "from": "0x0000000000000000000000000000000000000000",
    "value": { "$gte": "1000000000000000000" }
  }
}
```

## Common Testing Patterns

### 1. Database Testing Pattern

```rust
#[tokio::test]
async fn test_repository_method() {
    // Arrange
    let pool = test_db_pool().await;
    let repo = TenantRepository::new();
    let tenant = TenantBuilder::new().build();
    
    // Act
    let result = repo.create(&pool, tenant).await;
    
    // Assert
    assert!(result.is_ok());
    
    // Cleanup happens automatically with test transactions
}
```

### 2. Service Testing Pattern

```rust
#[tokio::test]
async fn test_service_method() {
    // Arrange
    let mut mock_repo = MockTenantRepository::new();
    mock_repo.expect_find_by_id()
        .with(eq(tenant_id))
        .returning(|_| Ok(Some(test_tenant())));
    
    let service = TenantService::new(mock_repo);
    
    // Act
    let result = service.get_tenant(tenant_id).await;
    
    // Assert
    assert!(result.is_ok());
}
```

### 3. API Testing Pattern

```rust
#[tokio::test]
async fn test_api_endpoint() {
    // Arrange
    let app = test_app().await;
    let token = create_test_jwt();
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/monitors")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(json!(test_monitor())))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

## Measuring Success

### Coverage Metrics That Matter

1. **Line Coverage**: Should be 95%+
2. **Branch Coverage**: All if/else paths tested
3. **Function Coverage**: Every public function tested
4. **Error Path Coverage**: All Result::Err cases tested

### What 95% Coverage Really Means

- Every happy path is tested
- Every error condition is tested
- Every edge case is considered
- Every public API is documented with tests
- Performance characteristics are verified

## Tools and Commands

```bash
# Run tests with coverage
make test-coverage

# Generate detailed HTML report
cargo llvm-cov --html

# Run specific test file
cargo test --test integration_tests

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Run property-based tests with more cases
PROPTEST_CASES=10000 cargo test

# Benchmark tests
cargo bench
```

## Conclusion

Achieving 95% test coverage isn't about gaming the metrics—it's about having confidence that:

1. The code works as intended
2. Edge cases are handled
3. Errors are propagated correctly
4. Performance is acceptable
5. Security is maintained

The OpenZeppelin Monitor project demonstrates that comprehensive testing requires:

- Thoughtful test organization
- Proper mocking infrastructure
- Reusable test utilities
- Focus on both success and failure paths
- Property-based testing for invariants

Our current 13% coverage only scratches the surface. To reach 95%, we need to fundamentally change our approach from "testing what's easy" to "testing what matters."
