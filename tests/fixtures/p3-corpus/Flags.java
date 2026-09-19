/**
 * The P3 3.3 flag corpus: one source, compiled several times with different legal flag sets, so the
 * same three shapes are read out of class files whose debug and parameter metadata differ.
 *
 * <p>The three members are the dimensions the flag matrix is about:
 *
 * <ul>
 *   <li>{@link #copied(int)} declares a local — its name is an ordinal without a
 *       {@code LocalVariableTable} and the source spelling with one;
 *   <li>{@link #choose(boolean, int)} takes a {@code boolean} parameter (P3-R5) and tests it;
 *   <li>{@link #counted(int)} reads and writes the class's own static field, which is how a run
 *       states a count the comparison can measure.
 * </ul>
 *
 * <p>It is compiled with {@code --release 8} under {@code -g:none}, {@code -g}, {@code
 * -g:lines,source} and {@code -parameters}, and once with {@code -source 8 -target 8} instead of
 * {@code --release 8}, so that "the flags differ, the shapes do not" is checked against real
 * class files rather than argued. See this directory's README for the exact commands and digests.
 */
public class Flags {

    static int probes;

    /** A read and a write of the class's own static field: the count a run can be measured by. */
    public static int counted(int seed) {
        probes = probes + 1;
        return seed + probes;
    }

    /** One local: named by the class file only when it carries a `LocalVariableTable`. */
    public static int copied(int seed) {
        int local = seed;
        return local;
    }

    /** The P3-R5 shape: a `boolean` parameter, which only the descriptor says is a `boolean`. */
    public static int choose(boolean flag, int seed) {
        if (flag) {
            return seed;
        }
        return seed - 1;
    }
}
