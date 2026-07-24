#!/usr/bin/env bash
# test-create-dev-subaccounts.sh — Mock test script for create-dev-subaccounts.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_SCRIPT="$SCRIPT_DIR/create-dev-subaccounts.sh"

echo "=== Testing create-dev-subaccounts.sh mock behavior ==="

run_mock_test() {
  local mock_status="$1"
  local expected_msg="$2"
  
  echo "Testing HTTP Status $mock_status..."
  
  # Create a temporary mock curl wrapper
  local tmp_bin_dir
  tmp_bin_dir=$(mktemp -d)
  
  cat <<EOF > "$tmp_bin_dir/curl"
#!/usr/bin/env bash
if [[ "\$*" == *"-w %{http_code}"* ]]; then
  echo -n "$mock_status"
  exit 0
fi
exec curl "\$@"
EOF
  chmod +x "$tmp_bin_dir/curl"
  
  local output
  output=$(PATH="$tmp_bin_dir:$PATH" bash "$TARGET_SCRIPT" test_user 2>&1 || true)
  
  rm -rf "$tmp_bin_dir"
  
  if echo "$output" | grep -q "$expected_msg"; then
    echo "PASS"
  else
    echo "FAIL: Expected message '$expected_msg' not found. Output:"
    echo "$output"
    exit 1
  fi
}

run_mock_test "200" "Funded successfully"
run_mock_test "400" "Friendbot returned 400"
run_mock_test "429" "Rate Limit Exceeded"
run_mock_test "500" "Server Failure"
run_mock_test "000" "Network Connection Error"

echo "=== All mock tests PASSED successfully ==="
