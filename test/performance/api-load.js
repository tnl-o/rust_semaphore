/**
 * Performance Tests: API Load Testing with k6
 * 
 * Install k6: 
 *   Windows: choco install k6
 *   macOS: brew install k6
 *   Linux: See https://k6.io/docs/getting-started/installation/
 * 
 * Run tests:
 *   k6 run api-load.js
 *   k6 run --vus 50 --duration 5m api-load.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');
const successfulLogins = new Counter('successful_logins');

// Test configuration
export const options = {
  // Scenario 1: Smoke test
  smoke: {
    executor: 'constant-vus',
    vus: 10,
    duration: '1m',
    gracefulStop: '30s',
  },
  
  // Scenario 2: Load test
  load: {
    executor: 'ramping-vus',
    startVUs: 0,
    stages: [
      { duration: '2m', target: 50 },   // Ramp up to 50 VUs
      { duration: '5m', target: 50 },   // Stay at 50 VUs
      { duration: '2m', target: 100 },  // Ramp up to 100 VUs
      { duration: '5m', target: 100 },  // Stay at 100 VUs
      { duration: '2m', target: 0 },    // Ramp down to 0 VUs
    ],
    gracefulRampDown: '30s',
    startTime: '1m', // Start after smoke test
  },
  
  // Scenario 3: Stress test
  stress: {
    executor: 'ramping-arrival-rate',
    startRate: 10,
    timeUnit: '1s',
    preAllocatedVUs: 100,
    maxVUs: 500,
    stages: [
      { duration: '2m', target: 50 },
      { duration: '5m', target: 100 },
      { duration: '2m', target: 200 },
      { duration: '5m', target: 0 },
    ],
    startTime: '15m', // Start after load test
  },
  
  thresholds: {
    http_req_duration: ['p(50)<100', 'p(95)<500', 'p(99)<1000'], // Response time thresholds
    http_req_failed: ['rate<0.01'], // Error rate < 1%
    errors: ['rate<0.1'], // Custom error rate < 10%
    response_time: ['p(95)<500'], // Custom response time
  },
  
  // Summary trend stats
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
};

// Test data
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const ADMIN_USERNAME = __ENV.ADMIN_USERNAME || 'admin';
const ADMIN_PASSWORD = __ENV.ADMIN_PASSWORD || 'password123';

// Shared session
let authToken = '';

// Setup: Login and get token
export function setup() {
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
    auth: ADMIN_USERNAME,
    password: ADMIN_PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  check(loginRes, {
    'setup: login status is 200': (r) => r.status === 200,
  });
  
  if (loginRes.status === 200) {
    const body = loginRes.json();
    authToken = body.token || '';
    successfulLogins.add(1);
  }
  
  return { authToken };
}

// Teardown: Logout
export function teardown(data) {
  if (data.authToken) {
    http.post(`${BASE_URL}/api/auth/logout`, null, {
      headers: { 
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${data.authToken}`,
      },
    });
  }
}

// Default test function
export default function (data) {
  const token = data.authToken;
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': token ? `Bearer ${token}` : '',
  };
  
  // Test 1: Health check endpoint
  testHealthCheck();
  sleep(1);
  
  // Test 2: Get projects list
  testGetProjects(headers);
  sleep(1);
  
  // Test 3: Get templates
  testGetTemplates(headers);
  sleep(1);
  
  // Test 4: Get tasks
  testGetTasks(headers);
  sleep(1);
}

// Individual test functions
function testHealthCheck() {
  const res = http.get(`${BASE_URL}/api/health`);
  
  const success = check(res, {
    'health: status is 200': (r) => r.status === 200,
    'health: response is OK': (r) => r.body === 'OK',
    'health: response time < 50ms': (r) => r.timings.duration < 50,
  });
  
  errorRate.add(!success);
  responseTime.add(res.timings.duration);
}

function testGetProjects(headers) {
  const res = http.get(`${BASE_URL}/api/projects`, { headers });
  
  const success = check(res, {
    'projects: status is 200': (r) => r.status === 200,
    'projects: response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  errorRate.add(!success);
  responseTime.add(res.timings.duration);
}

function testGetTemplates(headers) {
  const res = http.get(`${BASE_URL}/api/projects/1/templates`, { headers });
  
  const success = check(res, {
    'templates: status is 200': (r) => r.status === 200,
    'templates: response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  errorRate.add(!success);
  responseTime.add(res.timings.duration);
}

function testGetTasks(headers) {
  const res = http.get(`${BASE_URL}/api/projects/1/tasks`, { headers });
  
  const success = check(res, {
    'tasks: status is 200': (r) => r.status === 200,
    'tasks: response time < 300ms': (r) => r.timings.duration < 300,
  });
  
  errorRate.add(!success);
  responseTime.add(res.timings.duration);
}
