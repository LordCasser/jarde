// The baseline driver of the P3-R8 fixture: the committed original class is run in a controlled
// way, and this is what its own bytecode does — the observation the refused members rest on. The
// execution comparison compiles it beside the sample and asserts these exact lines (the
// `## What the comparison answered` table in README.md records them).
public class Baseline {
    public static void main(String[] args) {
        System.out.println("nestedLocal(7)=" + NestedEval.nestedLocal(7));
        System.out.println("nestedCall(3)=" + NestedEval.nestedCall(3));
        System.out.println("tick calls=" + NestedEval.calls);
    }
}
