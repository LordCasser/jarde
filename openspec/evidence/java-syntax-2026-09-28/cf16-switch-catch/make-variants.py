from pathlib import Path

here = Path(__file__).parent
base = (here / "SwitchCatchMinimal.java").read_text()
start = base.index("\tpublic String runTest(")
end = base.index("\n\t}\n}", start) + len("\n\t}")

partial = '''\tpublic String runTest(int testNumber, int excType) {
\t\tsb = new StringBuilder();
\t\tswitch (testNumber) {
\t\t\tcase 1:
\t\t\t\ttry {
\t\t\t\t\ttest1(excType);
\t\t\t\t} catch (IllegalArgumentException e) {
\t\t\t\t\tassertThat(excType).isEqualTo(2);
\t\t\t\t}
\t\t\t\tbreak;
\t\t\tcase 2:
\t\t\t\ttest2(excType);
\t\t\t\tbreak;
\t\t\tcase 3:
\t\t\t\ttest3(excType);
\t\t\t\tbreak;
\t\t}
\t\treturn sb.toString();
\t}'''

twr = '''\tpublic String runTest(int testNumber, int excType) {
\t\tsb = new StringBuilder();
\t\ttry (Resource ignored = new Resource()) {
\t\t\ttry {
\t\t\t\tswitch (testNumber) {
\t\t\t\t\tcase 1:
\t\t\t\t\t\ttest1(excType);
\t\t\t\t\t\tbreak;
\t\t\t\t\tcase 2:
\t\t\t\t\t\ttest2(excType);
\t\t\t\t\t\tbreak;
\t\t\t\t\tcase 3:
\t\t\t\t\t\ttest3(excType);
\t\t\t\t\t\tbreak;
\t\t\t\t}
\t\t\t} catch (IllegalArgumentException e) {
\t\t\t\tassertThat(excType).isEqualTo(2);
\t\t\t}
\t\t} catch (Exception e) {
\t\t\tthrow new AssertionError(e);
\t\t}
\t\treturn sb.toString();
\t}

\tprivate static final class Resource implements AutoCloseable {
\t\t@Override
\t\tpublic void close() {
\t\t}
\t}'''

for name, body in (("PartialSwitchCatch1", partial), ("TwrSwitchCatch", twr)):
    source = base[:start] + body + base[end:]
    source = source.replace("SwitchCatchMinimal", name)
    (here / f"{name}.java").write_text(source)

for selected in (2, 3):
    body = partial
    # Move the local handler from case 1 to the selected arm.
    case1 = "\t\t\tcase 1:\n\t\t\t\ttry {\n\t\t\t\t\ttest1(excType);\n\t\t\t\t} catch (IllegalArgumentException e) {\n\t\t\t\t\tassertThat(excType).isEqualTo(2);\n\t\t\t\t}\n\t\t\t\tbreak;"
    case_n = f"\t\t\tcase {selected}:\n\t\t\t\ttry {{\n\t\t\t\t\ttest{selected}(excType);\n\t\t\t\t}} catch (IllegalArgumentException e) {{\n\t\t\t\t\tassertThat(excType).isEqualTo(2);\n\t\t\t\t}}\n\t\t\t\tbreak;"
    body = body.replace(case1, "\t\t\tcase 1:\n\t\t\t\ttest1(excType);\n\t\t\t\tbreak;")
    body = body.replace(f"\t\t\tcase {selected}:\n\t\t\t\ttest{selected}(excType);\n\t\t\t\tbreak;", case_n)
    source = base[:start] + body + base[end:]
    source = source.replace("SwitchCatchMinimal", f"PartialSwitchCatch{selected}")
    (here / f"PartialSwitchCatch{selected}.java").write_text(source)
