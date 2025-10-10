# Business Telemetry Strategy - Number Guessing Game

## Executive Summary

This document outlines a comprehensive business telemetry strategy for the number guessing game application. The goal is to track key business metrics that provide insights into user engagement, game performance, API usage, and overall product health.

## Current State

The application already has:
- ✅ Structured logging via `tracing` framework
- ✅ Basic operational logs (game creation, guesses, completion)
- ✅ Error tracking
- ✅ HTTP request tracing (via tower-http TraceLayer)
- ✅ Stderr-based log output for monitoring tools

## Recommended Business Metrics

### 1. User Engagement Metrics

#### What to Track

| Metric | Description | Source |
|--------|-------------|--------|
| Games started | Total count, segmented by interface (API vs Web UI) | `create_game_api()`, `create_game_web()` |
| Game completion rate | % of games that finish vs. abandoned | Compare games created vs. games deleted |
| Win rate | % correct guesses vs. limit reached | `GuessResult::Correct` vs `GuessResult::LimitReached` |
| Average session duration | Time from game start to completion | Track created_at, calculate delta on completion |
| Abandoned games | Games still in DB after X hours | Query games table for old created_at timestamps |
| Games per user session | Sequential games played | Requires session tracking |

#### Business Value
- Understand user engagement patterns
- Identify drop-off points and abandonment reasons
- Measure product stickiness
- Optimize onboarding and user experience

#### Implementation Points
- **src/web.rs:create_game_api** (line 196) - Track game starts with interface type
- **src/web.rs:make_guess_api** (line 283) - Track completions/wins/losses
- **src/db.rs:create_game** (line 33) - Already has created_at via default
- **src/db.rs:make_guess_transactional** (line 225) - Calculate session duration on completion

---

### 2. Game Difficulty & Performance Metrics

#### What to Track

| Metric | Description | Calculation |
|--------|-------------|-------------|
| Average guesses to win | Mean number of guesses for successful games | `attempts` field in `GuessResult::Correct` |
| Guess efficiency | Actual vs. optimal guesses | `actual_guesses / log2(max - min + 1)` |
| Range size distribution | Histogram of (max - min) values | Track on game creation |
| Guess limit utilization | % of games using limits, distribution of limit values | Track `max_guesses` on game creation |
| Most common ranges | Top 10 min/max combinations | Group by (min, max) pairs |
| Win rate by range size | Correlation between range size and success | Segment wins by range buckets |
| Average guesses by range size | How range affects guess count | Segment guess counts by range buckets |

#### Business Value
- Understand game difficulty preferences
- Optimize default values for better UX
- Identify power users vs. casual players
- Guide feature development (e.g., difficulty presets)

#### Implementation Points
- **src/game.rs:make_guess** (line 144) - Track guess patterns and efficiency
- **src/web.rs:create_game_api** (line 196) - Track range configurations
- Add metric: `optimal_guesses = ceil(log2(max - min + 1))` for binary search baseline
- **src/db.rs:create_game** - Already captures min, max, max_guesses

---

### 3. API Performance & Usage Metrics

#### What to Track

| Metric | Description | Target/SLA |
|--------|-------------|------------|
| Request volume | Requests/minute, by endpoint | Monitor for capacity planning |
| Response times | p50, p95, p99 latencies | p95 < 100ms, p99 < 200ms |
| Error rates | By type: validation, not found, database | < 1% error rate |
| Concurrent games | Active games at any time | Monitor memory usage |
| Database query performance | Transaction duration, connection pool usage | p95 < 50ms |
| Successful vs. failed requests | HTTP 2xx vs. 4xx/5xx | > 99% success rate |
| Guess throughput | Guesses processed per second | Monitor for load testing |

#### Business Value
- Identify performance bottlenecks
- Capacity planning and scaling decisions
- SLA monitoring and alerting
- Database optimization opportunities

#### Implementation Points
- **src/web.rs** (line 92-95) - Already has tower-http tracing (DefaultMakeSpan, DefaultOnResponse)
- **src/db.rs:make_guess_transactional** (line 225) - Track transaction timing
- **src/web.rs:health_check** (line 181) - Monitor database health
- Add custom middleware for detailed metrics
- Track connection pool statistics via SQLx

---

### 4. Business Intelligence Metrics

#### What to Track

| Metric | Description | Analysis Period |
|--------|-------------|-----------------|
| Peak usage times | Hourly/daily/weekly patterns | Hourly bins, 7-day rolling |
| User behavior patterns | Sequential games, range experimentation | Session-based analysis |
| Feature adoption | Guess limits usage %, API vs. Web preference | Weekly comparison |
| Game lifecycle patterns | Time-to-first-guess, time-between-guesses | Aggregate statistics |
| Retention metrics | Returning users (requires user tracking) | Daily/Weekly/Monthly |
| Growth metrics | New games vs. previous period | Week-over-week, month-over-month |

#### Business Value
- Strategic decision-making
- Feature prioritization
- Growth tracking and forecasting
- Marketing and user acquisition insights

#### Implementation Points
- Aggregate from structured logs using log analysis tools
- Consider adding optional user identification (session cookies, optional auth)
- Time-series analysis of existing metrics
- Export to data warehouse for BI tools (e.g., BigQuery, Redshift)

---

## Priority Metrics (Quick Wins)

Implementation priority based on business value vs. effort:

| Metric | Priority | Effort | Business Value | Implementation Complexity |
|--------|----------|--------|----------------|---------------------------|
| Games created (by interface) | **High** | Low | High - Understand usage split | Add field to log event |
| Win rate vs. loss rate | **High** | Low | High - Measure engagement quality | Count result types |
| Average guesses to win | **High** | Low | High - Game difficulty insights | Aggregate attempts field |
| Game completion rate | **High** | Medium | High - Identify abandonment | Compare created vs. deleted |
| Error rate by type | **High** | Low | High - Quality monitoring | Aggregate error logs |
| Response time percentiles | Medium | Medium | High - Performance monitoring | Use existing tracing data |
| Range size distribution | Medium | Low | Medium - User preference insights | Histogram of (max-min) |
| Guess limit utilization | Medium | Low | Medium - Feature adoption | Count max_guesses usage |
| Concurrent games | Medium | Low | Medium - Capacity planning | Query games table count |
| Optimal vs. actual guesses | Low | Medium | Medium - Efficiency analysis | Calculate log2(range) |

---

## Implementation Approach

### Phase 1: Enhanced Event Logging (Low Effort, Immediate Value)

**Goal:** Add structured events to existing code using `tracing` framework

**No new infrastructure needed** - leverage existing tracing setup

#### Changes Required:

1. **Standardize Event Fields**
   - Use consistent field names across all events
   - Add structured fields for key metrics
   - Include context: interface type, range size, timing

2. **Key Events to Add/Enhance:**
   - `game.created` - min, max, max_guesses, interface (api/web), range_size
   - `game.guess` - game_id, guess_number, result, remaining_guesses
   - `game.completed` - game_id, result (won/lost), total_guesses, duration_ms, range_size
   - `game.abandoned` - game_id, guess_count, time_active
   - `api.request` - endpoint, status_code, duration_ms (already partially implemented)
   - `db.transaction` - operation, duration_ms, success

3. **Files to Modify:**
   - `src/web.rs` - Add interface type to logs (lines 196-281, 405-485)
   - `src/db.rs` - Add timing and performance metrics (lines 33-98, 225-392)
   - `src/game.rs` - Add game completion events (consider adding in web/db layer)

4. **Example Event Format:**
   ```rust
   info!(
       event = "game.created",
       game_id = %game_id,
       interface = "api",
       min = min,
       max = max,
       range_size = max - min,
       max_guesses = ?max_guesses,
       "Game created"
   );
   ```

**Output:** Structured logs in JSON format (can be added to tracing-subscriber config)

**Analysis:** Use log aggregation tools (ELK, Loki, CloudWatch Logs Insights)

---

### Phase 2: Metrics Aggregation (Medium Effort)

**Goal:** Real-time metrics collection and exposure

**Infrastructure:** In-memory metrics with Prometheus endpoint

#### Changes Required:

1. **Add Dependencies:**
   ```toml
   [dependencies]
   metrics = "0.23"
   metrics-exporter-prometheus = "0.15"
   ```

2. **Create Metrics Module (`src/metrics.rs`):**
   - Counter: `games_created_total{interface}`
   - Counter: `games_completed_total{result}` (won/lost)
   - Histogram: `game_duration_seconds`
   - Histogram: `guess_count{result}`
   - Histogram: `api_request_duration_seconds{endpoint,status}`
   - Gauge: `active_games_count`
   - Histogram: `db_transaction_duration_seconds{operation}`

3. **Instrument Code:**
   - Add metric calls alongside existing tracing logs
   - Minimal performance overhead
   - Non-blocking metric recording

4. **Expose Prometheus Endpoint:**
   - `/metrics` endpoint on health check server (port 8081)
   - Standard Prometheus text format

5. **Files to Modify:**
   - `src/lib.rs` - Add metrics module export
   - `src/metrics.rs` - New file for metrics definitions
   - `src/web.rs` - Instrument handlers with metrics
   - `src/db.rs` - Instrument database operations
   - `src/main.rs` - Initialize metrics exporter

**Output:** Prometheus-compatible metrics endpoint

**Visualization:** Grafana dashboards

---

### Phase 3: Analytics Pipeline (Higher Effort)

**Goal:** Business intelligence and historical analysis

**Infrastructure:** Time-series database and analytics platform

#### Components:

1. **Metrics Storage:**
   - **Option A:** Prometheus + Grafana (open-source, self-hosted)
   - **Option B:** Cloud provider metrics (CloudWatch, Azure Monitor, GCP Monitoring)
   - **Option C:** InfluxDB for high-cardinality data

2. **Log Analytics:**
   - **Option A:** ELK Stack (Elasticsearch, Logstash, Kibana)
   - **Option B:** Grafana Loki + Promtail
   - **Option C:** Cloud provider logs (CloudWatch Logs Insights, Azure Log Analytics)

3. **Business Intelligence:**
   - Export aggregated metrics to data warehouse
   - Create business dashboards (Metabase, Superset, Tableau)
   - Set up automated reporting

4. **Alerting:**
   - Error rate thresholds
   - Performance degradation alerts
   - Anomaly detection for usage patterns

#### Changes Required:

1. **Prometheus Configuration:**
   - Deploy Prometheus server
   - Configure scraping of `/metrics` endpoint
   - Set retention policies

2. **Grafana Dashboards:**
   - System health dashboard (errors, latency, throughput)
   - Business metrics dashboard (games, wins, engagement)
   - Database performance dashboard

3. **Log Export:**
   - Configure log shipping to aggregation platform
   - Set up JSON-formatted structured logs
   - Create log-based queries and dashboards

4. **Alerting Rules:**
   - Error rate > 1%
   - p95 latency > 200ms
   - Database connection pool exhaustion
   - Unusual drop in game creation rate

**Output:** Production-ready observability stack

---

## Metrics Taxonomy

### Structured Field Naming Convention

To ensure consistency across all telemetry events:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `event` | String | Event type identifier | "game.created", "game.completed" |
| `game_id` | u64 | Unique game identifier | 12345678901234567890 |
| `interface` | String | API or Web | "api", "web" |
| `min` | i32 | Minimum value in range | 1 |
| `max` | i32 | Maximum value in range | 100 |
| `range_size` | i32 | max - min | 99 |
| `max_guesses` | Option<u32> | Guess limit | Some(10), None |
| `guess_number` | u32 | Sequential guess count | 5 |
| `guess_value` | i32 | The guessed number | 42 |
| `result` | String | Guess/game result | "too_low", "too_high", "correct", "limit_reached" |
| `total_guesses` | u32 | Total guesses in completed game | 7 |
| `duration_ms` | u64 | Operation duration in milliseconds | 1234 |
| `optimal_guesses` | u32 | Binary search optimal | 7 |
| `efficiency` | f64 | actual / optimal | 1.2 |
| `endpoint` | String | API endpoint path | "/api/games", "/api/games/:id/guess" |
| `status_code` | u16 | HTTP status code | 200, 404, 500 |
| `error_type` | String | Error category | "validation", "not_found", "database" |

---

## Sample Queries and Dashboards

### Log Query Examples (for ELK/Loki/CloudWatch)

```
# Games created per hour
event="game.created" | stats count() by hour(timestamp)

# Win rate calculation
event="game.completed" | stats count() by result

# Average guesses to win
event="game.completed" AND result="correct" | stats avg(total_guesses)

# Range size distribution
event="game.created" | stats count() by range_size

# API error rate
status_code >= 400 | stats count() / total_requests * 100

# Slow database transactions
duration_ms > 100 AND event="db.transaction" | stats count() by operation
```

### Grafana Dashboard Panels

**System Health Dashboard:**
1. Request rate (requests/sec) - Time series
2. Error rate (%) - Time series with threshold alert
3. Response time (p50, p95, p99) - Time series
4. Active games count - Gauge
5. Database connection pool usage - Gauge

**Business Metrics Dashboard:**
1. Games created (total, by interface) - Time series + pie chart
2. Game outcomes (won/lost/abandoned) - Pie chart
3. Win rate over time - Time series
4. Average guesses to win - Time series
5. Range size distribution - Histogram
6. Games per hour (last 24h) - Heatmap

**Performance Dashboard:**
1. API endpoint latency - Heatmap by endpoint
2. Database query performance - Time series by operation
3. Transaction duration distribution - Histogram
4. Slow query log - Table view

---

## Data Privacy Considerations

### Current State
- No user identification or personal data collected
- Game IDs are random u64 values (not linked to users)
- No IP address logging in application code (may be in reverse proxy/load balancer)

### Recommendations
1. **Keep it anonymous** - Current approach is privacy-friendly
2. **Optional session tracking** - Use anonymous session tokens if needed
3. **No PII in logs** - Avoid logging any personally identifiable information
4. **GDPR compliance** - Current design already compliant (no personal data)
5. **Data retention** - Set appropriate retention policies (30-90 days for logs)

---

## Performance Impact

### Expected Overhead

| Phase | CPU Impact | Memory Impact | Latency Impact |
|-------|------------|---------------|----------------|
| Phase 1 (Enhanced logging) | < 1% | Minimal | < 1ms per request |
| Phase 2 (Metrics) | < 2% | ~10MB for metrics buffer | < 0.5ms per request |
| Phase 3 (Full stack) | < 3% | Depends on scrape interval | No additional impact |

### Optimization Tips
1. Use async logging (already implemented via `tracing`)
2. Sample high-frequency events if needed (e.g., 10% sampling for guesses)
3. Use batch exports for log shipping
4. Set appropriate metric cardinality limits
5. Use histograms instead of individual timing events

---

## Success Metrics for Telemetry Implementation

How to measure if the telemetry strategy is effective:

| Goal | Success Metric | Target |
|------|----------------|--------|
| Operational visibility | Mean time to detect (MTTD) issues | < 5 minutes |
| Data completeness | % of critical events tracked | 100% |
| Data quality | % of malformed/missing events | < 0.1% |
| Actionable insights | Decisions made from metrics | > 1 per sprint |
| Performance impact | p99 latency increase | < 5% |
| Team adoption | % of team viewing dashboards weekly | > 80% |

---

## Next Steps

### Recommended Path Forward

1. **Review and validate** this strategy with stakeholders
2. **Phase 1 implementation** (1-2 days effort):
   - Add structured event fields to existing logs
   - Test log output and parsing
   - Document event schema
3. **Set up log aggregation** (if not already done):
   - Choose platform (ELK, Loki, cloud provider)
   - Configure log shipping
   - Create initial queries/dashboards
4. **Iterate on Phase 2** (1 week effort):
   - Add metrics crate
   - Implement Prometheus endpoint
   - Create Grafana dashboards
5. **Monitor and refine**:
   - Gather feedback from team
   - Adjust metrics based on actual usage
   - Add custom alerts as needed

---

## Appendix: Related Files

### Files to Modify for Implementation

| File | Current Lines | Changes Needed |
|------|---------------|----------------|
| `src/main.rs` | 14-27 | Add metrics initialization |
| `src/web.rs` | 196-281, 283-401 | Add structured event logging |
| `src/web.rs` | 405-485, 487-622 | Add web interface tracking |
| `src/db.rs` | 33-98 | Add game creation metrics |
| `src/db.rs` | 225-392 | Add transaction timing |
| `src/game.rs` | 144-173 | Consider adding game events |
| `Cargo.toml` | dependencies | Add metrics, metrics-exporter-prometheus |

### New Files to Create

| File | Purpose |
|------|---------|
| `src/metrics.rs` | Centralized metrics definitions |
| `docs/telemetry.md` | Telemetry documentation |
| `docs/runbooks/alerts.md` | Alert response procedures |
| `dashboards/grafana/` | JSON dashboard definitions |
| `prometheus.yml` | Prometheus configuration |

---

## References

- Tracing crate documentation: https://docs.rs/tracing/
- Metrics crate documentation: https://docs.rs/metrics/
- Prometheus best practices: https://prometheus.io/docs/practices/naming/
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/specs/semconv/
- Grafana dashboard examples: https://grafana.com/grafana/dashboards/

---

**Document Version:** 1.0
**Last Updated:** 2025-10-10
**Status:** Proposed
**Owner:** Development Team
