# Performance Testing Guide

## Installation

### k6 (Recommended)

**Windows:**
```powershell
choco install k6
```

**macOS:**
```bash
brew install k6
```

**Linux:**
```bash
# Debian/Ubuntu
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

## Running Tests

### Basic Load Test
```bash
cd test/performance
k6 run api-load.js
```

### Custom Configuration
```bash
# 50 virtual users for 5 minutes
k6 run --vus 50 --duration 5m api-load.js

# With custom environment variables
BASE_URL=http://localhost:3000 ADMIN_USERNAME=admin k6 run api-load.js
```

### Cloud Execution (k6 Cloud)
```bash
k6 cloud api-load.js
```

## Test Scenarios

### 1. Smoke Test (1 minute)
- 10 virtual users
- Basic health check validation

### 2. Load Test (16 minutes)
- Ramp up from 0 to 50 VUs
- Sustain 50 VUs for 5 minutes
- Ramp up to 100 VUs
- Sustain 100 VUs for 5 minutes
- Ramp down

### 3. Stress Test (11 minutes)
- Ramp up arrival rate to 200 req/s
- Test system breaking point
- Maximum 500 VUs

## Metrics

### Response Time Thresholds
| Percentile | Threshold | Description |
|------------|-----------|-------------|
| p50 | <100ms | Median response time |
| p95 | <500ms | 95% of requests |
| p99 | <1000ms | 99% of requests |

### Error Rate
- Target: <1% failed requests
- Custom error rate: <10%

### Custom Metrics
- `errors`: Custom error rate
- `response_time`: Response time trend
- `successful_logins`: Counter of successful authentications

## Apache Bench Alternative

```bash
# 10000 requests, 100 concurrent
ab -n 10000 -c 100 http://localhost:3000/api/health

# 1000 requests to projects endpoint
ab -n 1000 -c 50 http://localhost:3000/api/projects
```

## wrk Alternative

```bash
# 12 threads, 400 connections, 30 seconds
wrk -t12 -c400 -d30s http://localhost:3000/api/health

# With custom script
wrk -t12 -c400 -d30s -s post.lua http://localhost:3000/api/auth/login
```

## Results Interpretation

### Good Performance
- p95 < 200ms for API endpoints
- Error rate < 0.1%
- Consistent response times under load

### Warning Signs
- p95 > 500ms
- Error rate > 1%
- Increasing response times over test duration
- Memory leaks (growing memory usage)

### Critical Issues
- p99 > 1000ms
- Error rate > 5%
- System crashes or hangs
- Database connection pool exhaustion

## CI/CD Integration

### GitHub Actions
```yaml
name: Performance Tests
on: [push]
jobs:
  k6:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install k6
        run: |
          curl -L https://github.com/k6io/k6/releases/latest/download/k6-linux-amd64.deb -o k6.deb
          sudo dpkg -i k6.deb
      - name: Run k6 test
        run: k6 run test/performance/api-load.js
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: k6-results
          path: test-results.json
```

## Monitoring During Tests

### System Metrics
- CPU usage: `top`, `htop`
- Memory: `free -m`, `vmstat`
- Disk I/O: `iostat`, `iotop`
- Network: `iftop`, `nethogs`

### Application Metrics
- Active connections
- Request queue length
- Database connection pool usage
- Cache hit/miss ratio

### Database Metrics
- Query execution time
- Lock wait time
- Transaction rate
- Deadlock count

## Troubleshooting

### High Response Times
1. Check database query performance
2. Review application logs for errors
3. Monitor system resources (CPU, memory, disk)
4. Check for network latency

### High Error Rate
1. Review application logs
2. Check database connection limits
3. Verify authentication token validity
4. Monitor rate limiting

### System Crashes
1. Check memory limits (OOM killer)
2. Review file descriptor limits
3. Check database connection pool
4. Monitor disk space
