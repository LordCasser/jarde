# `verify-baseline` v2 → v3

V2 remains unchanged. V3 only corrects `parse_javap`'s member-census boundary and versions the eventual output/schema to avoid overwriting earlier results.

The parser now ignores all text until the exact top-level class-body `{`, recognizes two-space member declarations only while inside that body, and stops at its exact top-level `}`. It requires the class body to close. This excludes constant-pool UTF-8 rows like `  #10 = Utf8 ...;` and class attributes after the body. Qualified constructor matching, flags tokenization, descriptors, and BCI parsing are unchanged from v2.

The updated parser was run in isolation against the four archived original raw javap texts and matched these exact censuses in both JDK legs:

- `InputFieldIncrement2`: field `a:Lem23/InputFieldIncrement2$A;` flags `0x0001`; `<init>()V` flags `0x0001`, BCIs `[0,1,4]`; `test1(I)V` flags `0x0001`, BCIs `[0,1,4,5,8,11,12,13,16]`; `test2(I)V` flags `0x0001`, BCIs `[0,1,4,5,8,9,10,13]`.
- `InputFieldIncrement2$A`: field `f:I` flags `0x0000`; `<init>()V` flags `0x0002`, BCIs `[0,1,4,5,6,9]`.

The same read-only check confirmed the actual original/JADX case census fields and field/method identity shapes. `py_compile` passed. The full acceptance verifier was not run.

Hashes:

- v2 verifier: `32438382c1a2e06ad68eb4f907ed9a9265dfb981738c1115295b7ad5449d3b2c`
- v3 verifier: `efead211ceb2ecde7540a4d0a8a0493f74a47009899be0dff603f8afcbfb9161`
- v3 README: `3305cd4aaa700221de5ae06fa19e144a687311b27fcb8867131c85d7f6b5e9a5`
