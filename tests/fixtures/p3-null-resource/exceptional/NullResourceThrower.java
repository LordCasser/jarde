/** External helper keeps the resource body itself a straight-line call. */
final class NullResourceThrower {
    static RuntimeException marker;
    static int bodyCalls;

    static void raise() {
        bodyCalls++;
        throw marker;
    }
}
