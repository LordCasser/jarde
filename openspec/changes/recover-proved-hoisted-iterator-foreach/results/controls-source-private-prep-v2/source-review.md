# Source and manifest review v2

{
  "schema": "hoisted-iterator-controls-source-review-v2",
  "cause": "source-prep-v1 Runner.java was updated to 4253 bytes / 4242d158…c6a5ac01 after manifest generation; the old manifest incorrectly recorded 4152 bytes and SHA-256 809bcdf0…f5346f0a. The baseline launcher correctly rejected the immutable source before collector invocation.",
  "java_sources": {
    "HoistedIteratorPositive.java": {
      "v1_bytes": 522,
      "v1_sha256": "5ed670809786a43863e019f86a6d7520d55ea987f3086b882f34932ae76eae50",
      "v2_bytes": 522,
      "v2_sha256": "5ed670809786a43863e019f86a6d7520d55ea987f3086b882f34932ae76eae50",
      "byte_identical": true,
      "diff": "empty"
    },
    "HoistedIteratorRefusal.java": {
      "v1_bytes": 577,
      "v1_sha256": "4f115eaa0f0d05e8bb41f12520e5e5eca24c9898f973c365f6eab222f9bffd66",
      "v2_bytes": 577,
      "v2_sha256": "4f115eaa0f0d05e8bb41f12520e5e5eca24c9898f973c365f6eab222f9bffd66",
      "byte_identical": true,
      "diff": "empty"
    },
    "Runner.java": {
      "v1_bytes": 4253,
      "v1_sha256": "4242d1581924041defdc25301cb8c740565ea77cb7cb46e9f444cc03c6a5ac01",
      "v2_bytes": 4253,
      "v2_sha256": "4242d1581924041defdc25301cb8c740565ea77cb7cb46e9f444cc03c6a5ac01",
      "byte_identical": true,
      "diff": "empty"
    }
  },
  "manifest_change": {
    "v1_runner": {
      "bytes": 4152,
      "sha256": "809bcdf022f096c620b3366adaec531759f603bf672149ee2abc55ecf5346f0a"
    },
    "v2_runner": {
      "class": "cf11.Runner",
      "source": "Runner.java",
      "bytes": 4253,
      "sha256": "4242d1581924041defdc25301cb8c740565ea77cb7cb46e9f444cc03c6a5ac01"
    }
  },
  "preserved_v1_preflight": {
    "execution.json": {
      "path": "/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v1/execution.json",
      "bytes": 817,
      "sha256": "23f7ec9ac7304d15b0c728105e7a56ec1b0b3e24c95deb09ffc3466dd8e1f130"
    },
    "stdout.raw": {
      "path": "/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v1/stdout.raw",
      "bytes": 0,
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    "stderr.raw": {
      "path": "/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v1/stderr.raw",
      "bytes": 143,
      "sha256": "dbeb353d84a83e446b6f84501a25090819f84f01bc04f7e856530486da63ad15"
    },
    "invocation_exit_code": 1,
    "collector_started": false
  },
  "v2_action": "regenerate manifest from the exact v1 Java sources; no Java source edit"
}
