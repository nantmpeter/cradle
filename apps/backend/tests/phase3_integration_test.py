#!/usr/bin/env python3
"""
Phase 3 Integration Tests for rust-admin Dashboard + System Settings features.

Covers:
  1. Dashboard Stats API (GET /api/dashboard/stats)
  2. Dashboard Settings API (GET /api/dashboard/settings)
  3. Regression smoke tests (login, roles list)

Backend must be running on http://localhost:8080.
Run: cd apps/backend && python3 tests/phase3_integration_test.py
"""

import json
import sys
import time
import traceback
import unittest

import requests

BASE_URL = "http://localhost:8080"
ADMIN_EMAIL = "admin@example.com"
ADMIN_PASSWORD = "Test1234"
USER_EMAIL = "phase2test@test.com"
USER_PASSWORD = "Test1234"

# ─── Helpers ─────────────────────────────────────────────────────────────────


def api(method, path, token=None, json_body=None, expected_status=None):
    """Send an HTTP request and return (status_code, response_json)."""
    url = f"{BASE_URL}{path}"
    headers = {}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    if json_body is not None:
        headers["Content-Type"] = "application/json"

    resp = requests.request(method, url, headers=headers, json=json_body, timeout=15)
    body = {}
    try:
        body = resp.json()
    except Exception:
        pass

    if expected_status is not None and resp.status_code != expected_status:
        raise AssertionError(
            f"Expected {expected_status}, got {resp.status_code} — "
            f"path={method} {path} body={body}"
        )
    return resp.status_code, body


def login(email, password, retries=3, delay=1.0):
    """Login with retry on rate-limit. Return (access_token, refresh_token, full_response)."""
    for attempt in range(retries):
        s, b = api("POST", "/api/auth/login", json_body={"email": email, "password": password})
        if s == 200:
            return b.get("access_token"), b.get("refresh_token"), b
        if s == 429:
            time.sleep(delay)
            continue
        raise AssertionError(f"Login failed ({s}): {b}")
    raise AssertionError(f"Login rate-limited after {retries} retries")


def get_admin_token():
    """Get a fresh admin token."""
    t, _, _ = login(ADMIN_EMAIL, ADMIN_PASSWORD)
    return t


def get_user_token():
    """Get a fresh user token."""
    t, _, _ = login(USER_EMAIL, USER_PASSWORD)
    return t


def wait_for_rate_limit_reset(seconds=7):
    """Wait for the login rate limiter to replenish tokens."""
    time.sleep(seconds)


# ─── Test Classes ────────────────────────────────────────────────────────────


class TestDashboardStats(unittest.TestCase):
    """Tests for GET /api/dashboard/stats — statistics with field-level permission filtering."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_admin_token()
        cls.user_token = get_user_token()

    # ── Superadmin sees all fields ──────────────────────────────────────────

    def test_01_stats_as_superadmin(self):
        """Superadmin sees all top-level fields: users, roles, today_logins, recent_audit_logs."""
        s, b = api("GET", "/api/dashboard/stats", token=self.admin_token, expected_status=200)
        self.assertIn("users", b, "Missing 'users' field in stats response")
        self.assertIn("roles", b, "Missing 'roles' field in stats response")
        self.assertIn("today_logins", b, "Missing 'today_logins' field in stats response")
        self.assertIn("recent_audit_logs", b, "Missing 'recent_audit_logs' field in stats response")

    def test_02_stats_users_fields(self):
        """Users field contains total, active, disabled; total = active + disabled."""
        s, b = api("GET", "/api/dashboard/stats", token=self.admin_token, expected_status=200)
        users = b["users"]
        self.assertIn("total", users, "Missing 'total' in users")
        self.assertIn("active", users, "Missing 'active' in users")
        self.assertIn("disabled", users, "Missing 'disabled' in users")
        self.assertIsInstance(users["total"], int)
        self.assertIsInstance(users["active"], int)
        self.assertIsInstance(users["disabled"], int)
        self.assertEqual(
            users["total"], users["active"] + users["disabled"],
            f"users.total ({users['total']}) != active ({users['active']}) + disabled ({users['disabled']})"
        )

    def test_03_stats_roles_fields(self):
        """Roles contains total and distribution array; each element has role_name and count."""
        s, b = api("GET", "/api/dashboard/stats", token=self.admin_token, expected_status=200)
        roles = b["roles"]
        self.assertIn("total", roles, "Missing 'total' in roles")
        self.assertIsInstance(roles["total"], int)
        self.assertIn("distribution", roles, "Missing 'distribution' in roles")
        self.assertIsInstance(roles["distribution"], list)
        for item in roles["distribution"]:
            self.assertIn("role_name", item, "Distribution item missing 'role_name'")
            self.assertIn("count", item, "Distribution item missing 'count'")

    def test_04_stats_today_logins(self):
        """today_logins is a non-negative integer."""
        s, b = api("GET", "/api/dashboard/stats", token=self.admin_token, expected_status=200)
        self.assertIn("today_logins", b)
        self.assertIsInstance(b["today_logins"], int)
        self.assertGreaterEqual(b["today_logins"], 0)

    def test_05_stats_recent_audit_logs(self):
        """recent_audit_logs is an array with id/action/user_email/created_at, max 5 entries."""
        s, b = api("GET", "/api/dashboard/stats", token=self.admin_token, expected_status=200)
        logs = b["recent_audit_logs"]
        self.assertIsInstance(logs, list)
        self.assertLessEqual(len(logs), 5, "recent_audit_logs should have at most 5 entries")
        for entry in logs:
            self.assertIn("id", entry, "Audit log entry missing 'id'")
            self.assertIn("action", entry, "Audit log entry missing 'action'")
            self.assertIn("user_email", entry, "Audit log entry missing 'user_email'")
            self.assertIn("created_at", entry, "Audit log entry missing 'created_at'")

    # ── Auth requirement ────────────────────────────────────────────────────

    def test_06_stats_requires_auth(self):
        """No token returns 401."""
        s, b = api("GET", "/api/dashboard/stats")
        self.assertEqual(s, 401, f"Expected 401, got {s}")

    # ── Regular user field-level filtering ──────────────────────────────────

    def test_07_stats_regular_user_filtered(self):
        """Regular user (only users:read) sees users and today_logins but roles.distribution=null and recent_audit_logs empty."""
        s, b = api("GET", "/api/dashboard/stats", token=self.user_token, expected_status=200)
        # Should see users field
        self.assertIn("users", b, "Regular user should see 'users'")
        self.assertIn("total", b["users"])
        # Should see today_logins
        self.assertIn("today_logins", b, "Regular user should see 'today_logins'")
        # roles.distribution should be null (filtered)
        self.assertIn("roles", b, "Regular user should see 'roles' object")
        self.assertIsNone(
            b["roles"].get("distribution"),
            f"Regular user should NOT see roles.distribution, got: {b['roles'].get('distribution')}"
        )
        # recent_audit_logs should be empty array
        self.assertIn("recent_audit_logs", b)
        self.assertEqual(
            b["recent_audit_logs"], [],
            "Regular user should see empty recent_audit_logs"
        )

    def test_08_stats_roles_total_visible_to_all(self):
        """roles.total is visible to all authenticated users."""
        s, b = api("GET", "/api/dashboard/stats", token=self.user_token, expected_status=200)
        self.assertIn("roles", b)
        self.assertIn("total", b["roles"])
        self.assertIsInstance(b["roles"]["total"], int)
        self.assertGreaterEqual(b["roles"]["total"], 0)


class TestDashboardSettings(unittest.TestCase):
    """Tests for GET /api/dashboard/settings — system settings with dashboard:read permission."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_admin_token()
        cls.user_token = get_user_token()

    # ── Superadmin full access ──────────────────────────────────────────────

    def test_09_settings_as_superadmin(self):
        """Superadmin sees system, database, and current_user sections."""
        s, b = api("GET", "/api/dashboard/settings", token=self.admin_token, expected_status=200)
        self.assertIn("system", b, "Missing 'system' in settings response")
        self.assertIn("database", b, "Missing 'database' in settings response")
        self.assertIn("current_user", b, "Missing 'current_user' in settings response")

    def test_10_settings_system_info(self):
        """System field contains name, version, uptime_seconds; version is not empty."""
        s, b = api("GET", "/api/dashboard/settings", token=self.admin_token, expected_status=200)
        system = b["system"]
        self.assertIn("name", system, "Missing 'name' in system")
        self.assertIn("version", system, "Missing 'version' in system")
        self.assertIn("uptime_seconds", system, "Missing 'uptime_seconds' in system")
        self.assertTrue(
            len(system["version"]) > 0,
            f"Version should not be empty, got: '{system['version']}'"
        )

    def test_11_settings_database_info(self):
        """Database field contains status='connected' and max_connections."""
        s, b = api("GET", "/api/dashboard/settings", token=self.admin_token, expected_status=200)
        database = b["database"]
        self.assertIn("status", database, "Missing 'status' in database")
        self.assertEqual(
            database["status"], "connected",
            f"Expected database status='connected', got '{database['status']}'"
        )
        self.assertIn("max_connections", database, "Missing 'max_connections' in database")

    def test_12_settings_current_user(self):
        """current_user contains email, role, permissions array, and created_at."""
        s, b = api("GET", "/api/dashboard/settings", token=self.admin_token, expected_status=200)
        user = b["current_user"]
        self.assertIn("email", user, "Missing 'email' in current_user")
        self.assertIn("role", user, "Missing 'role' in current_user")
        self.assertIn("permissions", user, "Missing 'permissions' in current_user")
        self.assertIn("created_at", user, "Missing 'created_at' in current_user")
        self.assertIsInstance(user["permissions"], list, "permissions should be a list")

    # ── Auth and permission checks ─────────────────────────────────────────

    def test_13_settings_requires_auth(self):
        """No token returns 401."""
        s, b = api("GET", "/api/dashboard/settings")
        self.assertEqual(s, 401, f"Expected 401, got {s}")

    def test_14_settings_regular_user_forbidden(self):
        """Regular user without dashboard:read permission gets 403."""
        s, b = api("GET", "/api/dashboard/settings", token=self.user_token)
        self.assertEqual(
            s, 403,
            f"Expected 403 for regular user without dashboard:read, got {s}: {b}"
        )

    # ── Uptime validation ──────────────────────────────────────────────────

    def test_15_settings_uptime_positive(self):
        """uptime_seconds should be > 0 for a running server."""
        s, b = api("GET", "/api/dashboard/settings", token=self.admin_token, expected_status=200)
        uptime = b["system"]["uptime_seconds"]
        self.assertIsInstance(uptime, (int, float), f"uptime_seconds should be numeric, got {type(uptime)}")
        self.assertGreater(uptime, 0, f"uptime_seconds should be > 0, got {uptime}")


class TestRegressionSmoke(unittest.TestCase):
    """Quick smoke tests to verify Phase 1 and Phase 2 features still work."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_admin_token()

    def test_16_phase1_smoke_login(self):
        """Phase 1 regression: login still works normally."""
        access, refresh, body = login(ADMIN_EMAIL, ADMIN_PASSWORD)
        self.assertIsNotNone(access, "access_token should not be None")
        self.assertIsNotNone(refresh, "refresh_token should not be None")

    def test_17_phase2_smoke_roles_list(self):
        """Phase 2 regression: roles list still works (GET /api/roles)."""
        s, b = api("GET", "/api/roles", token=self.admin_token, expected_status=200)
        self.assertIsInstance(b, list)
        names = [r["name"] for r in b]
        self.assertIn("superadmin", names, "superadmin role should exist")


# ─── Custom Runner ───────────────────────────────────────────────────────────

def run_tests():
    """Run all test classes and print a report."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    test_classes = [
        TestDashboardStats,
        TestDashboardSettings,
        TestRegressionSmoke,
    ]

    for cls in test_classes:
        suite.addTests(loader.loadTestsFromTestCase(cls))

    total = suite.countTestCases()
    print("=" * 72)
    print(f"  Phase 3 Integration Tests — {total} tests")
    print("=" * 72)
    print()

    result = unittest.TextTestRunner(verbosity=2, stream=sys.stderr).run(suite)

    passed = result.testsRun - len(result.failures) - len(result.errors) - len(result.skipped)
    failed = len(result.failures)
    errors = len(result.errors)
    skipped = len(result.skipped)

    print()
    print("=" * 72)
    print("  PHASE 3 TEST REPORT")
    print("=" * 72)
    print(f"  Total: {result.testsRun} | Passed: {passed} | "
          f"Failed: {failed} | Errors: {errors} | Skipped: {skipped}")
    print()

    if result.failures:
        print("  FAILED TESTS:")
        for test, traceback_str in result.failures:
            last_line = traceback_str.strip().split("\n")[-1] if traceback_str else "N/A"
            print(f"    - {test}: {last_line}")
        print()

    if result.errors:
        print("  ERROR TESTS:")
        for test, traceback_str in result.errors:
            last_line = traceback_str.strip().split("\n")[-1] if traceback_str else "N/A"
            print(f"    - {test}: {last_line}")
        print()

    # Routing decision
    if failed == 0 and errors == 0:
        print("  Routing: ALL PASSED → No action needed")
    else:
        print("  Routing: FAILURES/ERRORS FOUND → Review required")
        for test, tb in result.failures:
            print(f"\n  [FAIL] {test}")
            print(f"  {tb}")
        for test, tb in result.errors:
            print(f"\n  [ERROR] {test}")
            print(f"  {tb}")

    print()
    print("=" * 72)
    return failed == 0 and errors == 0


if __name__ == "__main__":
    # Check backend availability
    try:
        requests.get(f"{BASE_URL}/api/auth/login", timeout=3)
    except requests.ConnectionError:
        print("ERROR: Backend not running on", BASE_URL)
        print("Start it with: cd apps/backend && cargo run")
        sys.exit(1)

    success = run_tests()
    sys.exit(0 if success else 1)
