# Security TODO List

This document outlines security improvements needed for the Number Guessing Game application.

## Server Security

- [ ] **Restrict Binding Interface**: Change binding from `0.0.0.0` to `127.0.0.1` in `web.rs:75` unless external access is required
- [ ] **Implement CORS Policy**: Add proper CORS configuration to restrict which origins can access the API

## Resource Protection

- [ ] **Implement Game Limits**: Add a maximum limit to the number of games stored in the HashMap
- [ ] **Add Game Timeouts**: Implement a mechanism to expire games after a period of inactivity
- [ ] **Add Rate Limiting**: Implement rate limiting for API endpoints to prevent abuse

## Input Validation

- [x] **Add Range Validation**: Add reasonable limits on the min/max range values
  - ✅ Implemented validation for non-negative values (min >= 0, max >= 0)
  - ✅ Added maximum limit of 1,000,000 for both min and max values
  - ✅ Added overflow protection in range calculations
  - ✅ Added comprehensive validation in CLI, web API, and web UI
- [ ] **Validate Game IDs**: Ensure game IDs match expected formats before lookup
- [ ] **Improve Error Handling**: Replace `expect()` calls with more graceful error handling

## Web Security

- [ ] **Consider Local Scripts**: Consider bundling HTMX with the application rather than using the CDN
- [ ] **Add Security Headers**: Implement Content Security Policy (CSP) and other security headers
- [ ] **Use Template Escaping**: Use proper HTML templating rather than string concatenation for HTML generation

## General Recommendations

- [ ] **Add Request Logging**: Implement proper logging for security events and debugging
- [ ] **Add Authentication**: If deployed publicly, consider adding authentication
- [ ] **Add Game Session Expiry**: Implement a cleanup mechanism for abandoned game sessions

## Implementation Priority

1. **High Priority**
   - Restrict binding interface to localhost
   - Implement game limits and timeouts
   - Add range validation

2. **Medium Priority**
   - Implement CORS policy
   - Add rate limiting
   - Improve error handling

3. **Lower Priority**
   - Add security headers
   - Consider local script bundling
   - Add authentication (if needed publicly)