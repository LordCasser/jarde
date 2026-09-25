/**
 * Java 8 source for the minimal boolean-array store fixture.
 *
 * The source is intentionally int[]: javac cannot express a raw int assignment to boolean[].
 * patch_classfiles.py changes only the method descriptor and the two array opcodes.
 */
public class RawBool {
    public static int put(int[] array, int index, int value) {
        array[index] = value;
        return array[index];
    }
}
