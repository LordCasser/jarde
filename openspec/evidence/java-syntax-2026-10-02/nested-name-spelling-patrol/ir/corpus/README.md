corpus scan: 452 class files under tests/**, both legs (mainline stash-before vs this change),
release binaries, single-class policy, class name read from each file own this_class.

- 448 artifacts byte-identical.
- 7 artifacts exit non-zero in BOTH legs equally (pre-existing unspellable/budget fixtures).
- 4 artifacts differ, exactly one line each, all nested-form conversions:
  diff -r /tmp/nn-corpus-before/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericObject.txt /tmp/nn-corpus-after/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericObject.txt
  15c15
  <     public static java.lang.Object make(matrix.Outer$A arg0, int arg1) {
  ---
  >     public static java.lang.Object make(matrix.Outer.A arg0, int arg1) {
  diff -r /tmp/nn-corpus-before/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericTyped.txt /tmp/nn-corpus-after/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericTyped.txt
  15c15
  <     public static matrix.Outer$A$Generic make(matrix.Outer$A arg0, int arg1) {
  ---
  >     public static matrix.Outer.A.Generic make(matrix.Outer.A arg0, int arg1) {
  diff -r /tmp/nn-corpus-before/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlain.txt /tmp/nn-corpus-after/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlain.txt
  15c15
  <     public static java.lang.Object make(matrix.Outer$A arg0, int arg1) {
  ---
  >     public static java.lang.Object make(matrix.Outer.A arg0, int arg1) {
  diff -r /tmp/nn-corpus-before/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlainRaw.txt /tmp/nn-corpus-after/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlainRaw.txt
  14c14
  <     public static java.lang.Object make(matrix.Outer$A arg0, int arg1) {
  ---
  >     public static java.lang.Object make(matrix.Outer.A arg0, int arg1) {
