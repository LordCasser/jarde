# Legacy subroutine clone condition boundary

Private v1 has multiple ret sites and is not a JVM-valid compound negative input: root JDK8 rejects Multiple returns to single jsr (cycle-negative-validation-root-v1/observed-bytecode-v1 command20). Preserved unchanged.

Private v2 joins both outcomes to a single ret5@35. Root independently executes javap/javac/Class.forName + getDeclaredMethods with -Xverify:all on JDK8/JDK23: all six commands exit0 without guard stop. Target test is never executed. Exact class SHA74d632baa95485b7233476984a56598b9689cfa1552f965b5a1dd87704ad1525. Rust reader→proof negative test is pending; no clone/path recovery/refusal claim yet.

Private Rust test v1 was rejected before application: its BLAKE3 assertion accidentally compared a SHA256 string, and Budget usage read conflicted with the mutable Walker borrow. Root independently executes the existing frozen digest oracle (SHAe994dba8124b0f5c6c782b163c0eeb0b5bdc284a66a766a4404310d266be5be7) and obtains BLAKE3 af3b18f31c433684f1c685c62a6ce11b540e8304f158e576b8776bc5d76a5a4b /2662 bytes, exit0 and no guard stop. Await corrected private test.
