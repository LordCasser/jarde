// The baseline driver of the P3-R9 fixture: the committed original classes are run in a controlled
// way, and this is what their own bytecode does — the observations the refused members rest on.
// `RefusedCast.calls` is the fixture's own counter: `External`'s static initializer moves it once,
// so the line after `fieldCast()` states that the read really can run that initializer, and the
// last line states the three `tick()` calls the executed controls make. The execution comparison
// compiles it beside the sample and asserts these exact lines (the `## What the comparison
// answered` table in README.md records them).
public class Baseline {
    public static void main(String[] args) {
        System.out.println("fieldCast()=" + RefusedCast.fieldCast());
        System.out.println("External initializations=" + RefusedCast.calls);
        System.out.println("instanceCast(new External())=" + RefusedCast.instanceCast(new External()));
        try {
            RefusedCast.instanceCast(null);
            System.out.println("instanceCast(null)=returned");
        } catch (NullPointerException expected) {
            System.out.println("instanceCast(null)=java.lang.NullPointerException");
        }
        System.out.println("chainCast()=" + RefusedCast.chainCast());
        System.out.println("leftRead()=" + RefusedCast.leftRead());
        System.out.println("rightRead()=" + RefusedCast.rightRead());
        System.out.println("tick calls=" + RefusedCast.calls);
    }
}
