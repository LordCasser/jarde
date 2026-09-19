/**
 * The P3 3.3 missing-dependency corpus: one member that names a class this fixture does **not**
 * ship, and one that needs nothing but its own parameter.
 *
 * <p>It is compiled against the stub in `absent-Library-stub/`, and only this class's own bytes are
 * kept: the stub is not part of the fixture, so `absent.Library` is unresolvable at run time, which
 * is what "a class whose dependency is missing" means for a caller that reads one class file. The
 * recovery layer reads one class file's bytes and has no classpath, so the question this sample asks
 * is whether it presents, refuses or quotes the call — and whether the answer can even be turned
 * back into a compilation unit, which needs the absent type.
 *
 * <p>`plain` is the control in the other direction: a member of the same sample that names nothing
 * outside the fixture, so the same class file is comparable by execution.
 */
public class MissingDependency {

    /** Names a class this fixture does not ship. */
    public static int viaAbsentLibrary(int seed) {
        return absent.Library.grow(seed);
    }

    /** Needs nothing but its own parameter: the executable control of this sample. */
    public static int plain(int seed) {
        return seed + 1;
    }
}
