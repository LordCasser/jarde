# Unicode exact-initializer assertion repair (private review)

The proposed patch changes only two expected declaration strings in `tests/recover_unicode_identifiers.rs`; it does not relax matching to a prefix. The test still checks the CJK field/method/nested-class/local names, absence of aliases/refusal markers, and the two-leg source/runtime round-trip assertions elsewhere in the file.

Captured evidence is from CI run `38017574260`, head `41b8e17516b21091e68f3c9f03fa5bd6d041cea6`, stable job `114111169840`, in `ci-41b8-failure-v1`. The stable log at lines 4733 and 4735 prints these exact declarations:

```java
static int 变量 = 1;
static java.lang.String 描述 = new java.lang.StringBuilder().append("变量=").append(UT.变量).toString();
```

The fixture source `tests/fixtures/recover-unicode-identifiers/ut/UT.java` initializes `变量` to `1` and initializes `描述` from `"变量=" + 变量`. The failure at `tests/recover_unicode_identifiers.rs:292` is therefore a stale empty-declaration expectation, not a loss of Unicode identity. The observed binary summary is 3 passed / 1 failed; the captured failure is the exact static declaration assertion.

## Nearby static-initializer evidence and residual risk

The same stable job reports:

- `class_static_initializer_projection.rs` (log line 1258; summary line 1268): `stable / test and specification	UNKNOWN STEP	2026-10-10T02:46:49.1118139Z test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.12s`
- `p3_final_static.rs` (log line 2873; summary line 2882): `stable / test and specification	UNKNOWN STEP	2026-10-10T02:49:20.6683279Z test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.16s`
- `recover_static_generic_field_init_text.rs` (log line 4689; summary line 4697): `stable / test and specification	UNKNOWN STEP	2026-10-10T02:52:07.0039139Z test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.32s`
- `recover_unicode_identifiers.rs` (log line 4716; summary line 4781): `stable / test and specification	UNKNOWN STEP	2026-10-10T02:52:09.9458363Z test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.88s`

A narrow source scan found other blank static-field assertions, but they refer to fields with no initializer in their test inputs (`NoInitializer.value`, nested-class reference fields) or to `final`-static-specific behavior; they are not stale expected spellings for initialized nonfinal fields. The Unicode test’s two exact expectations are the only matching initialized-field stale assertions identified. This is a source/log audit, not a claim that unexecuted tests pass; root should rely on the subsequent CI run for final verification.

Patch SHA-256: `daec9c976dc66b9fefcf8d360ea5782338b3660ac12ebb788a0c69cbb251b3fa`
